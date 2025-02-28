use shared::types::action::RelativeDirection;
use shared::types::cardinal_direction::CardinalDirection;

pub struct Map {
    pub player_position: (isize, isize),
    pub grid: Vec<Vec<String>>,
    pub visits: Vec<Vec<u32>>,
    pub current_cardinal_direction: CardinalDirection,
}

impl Map {
    pub fn new(
        initial_grid: &Vec<Vec<String>>,
        current_cardinal_direction: CardinalDirection,
    ) -> Map {
        let grid: Vec<Vec<String>> = initial_grid.clone();
        let view_size: usize = grid.len();
        let center: (isize, isize) = (
            view_size as isize / 2,
            if view_size > 0 {
                grid[0].len() as isize / 2
            } else {
                0
            },
        );
        let visits: Vec<Vec<u32>> = vec![vec![0; grid[0].len()]; grid.len()];
        Map {
            player_position: center,
            grid,
            visits,
            current_cardinal_direction,
        }
    }

    pub fn merge_radar_view(&mut self, new_view: &Vec<Vec<String>>, direction: CardinalDirection) {
        self.current_cardinal_direction = direction;
        let (row_offset, column_offset) = match direction {
            CardinalDirection::North => (-2, 0),
            CardinalDirection::South => (2, 0),
            CardinalDirection::East => (0, 2),
            CardinalDirection::West => (0, -2),
        };

        let candidate: (isize, isize) = (
            self.player_position.0 + row_offset,
            self.player_position.1 + column_offset,
        );

        let (new_grid, effective_position, overall_top, overall_left, new_rows, new_cols) =
            Map::merge_radar_view_to_map_grid(&self.grid, new_view, candidate);
        self.grid = new_grid;
        self.player_position = effective_position;

        self.visits =
            Map::merge_visits(&self.visits, overall_top, overall_left, new_rows, new_cols);
    }

    fn merge_radar_view_to_map_grid(
        saved: &Vec<Vec<String>>,
        new_view: &Vec<Vec<String>>,
        merge_center: (isize, isize),
    ) -> (Vec<Vec<String>>, (isize, isize), isize, isize, usize, usize) {
        let view_size: usize = new_view.len();
        let half: usize = view_size / 2;

        let new_top: isize = merge_center.0 - half as isize;
        let new_left: isize = merge_center.1 - half as isize;
        let new_bottom: isize = new_top + view_size as isize - 1;
        let new_right: isize = new_left + view_size as isize - 1;

        let saved_rows: isize = saved.len() as isize;
        let saved_cols: isize = if saved.is_empty() {
            0
        } else {
            saved[0].len() as isize
        };

        let overall_top: isize = 0.min(new_top);
        let overall_left: isize = 0.min(new_left);
        let overall_bottom: isize = (saved_rows - 1).max(new_bottom);
        let overall_right: isize = (saved_cols - 1).max(new_right);

        let new_rows: usize = (overall_bottom - overall_top + 1) as usize;
        let new_cols: usize = (overall_right - overall_left + 1) as usize;

        let mut merged: Vec<Vec<String>> = vec![vec![String::from("#"); new_cols]; new_rows];

        let offset_row: isize = -overall_top;
        let offset_col: isize = -overall_left;
        for i in 0..(saved_rows as usize) {
            for j in 0..(saved_cols as usize) {
                merged[i + offset_row as usize][j + offset_col as usize] = saved[i][j].clone();
            }
        }

        let merge_top: usize = (new_top - overall_top) as usize;
        let merge_left: usize = (new_left - overall_left) as usize;
        for i in 0..view_size {
            for j in 0..view_size {
                let cell: String = new_view[i][j].clone();
                if cell != "#" {
                    merged[merge_top + i][merge_left + j] = cell;
                }
            }
        }

        for row in merged.iter_mut() {
            for cell in row.iter_mut() {
                if cell == "P" {
                    *cell = String::from(" ");
                }
            }
        }

        let effective_position: (isize, isize) =
            (merge_center.0 - overall_top, merge_center.1 - overall_left);

        (
            merged,
            effective_position,
            overall_top,
            overall_left,
            new_rows,
            new_cols,
        )
    }

    fn merge_visits(
        saved: &Vec<Vec<u32>>,
        overall_top: isize,
        overall_left: isize,
        new_rows: usize,
        new_cols: usize,
    ) -> Vec<Vec<u32>> {
        let mut merged: Vec<Vec<u32>> = vec![vec![0; new_cols]; new_rows];
        let saved_rows: isize = saved.len() as isize;
        let saved_cols: isize = if saved.is_empty() {
            0
        } else {
            saved[0].len() as isize
        };
        let offset_row: isize = -overall_top;
        let offset_col: isize = -overall_left;
        for i in 0..saved_rows {
            for j in 0..saved_cols {
                merged[(i + offset_row) as usize][(j + offset_col) as usize] =
                    saved[i as usize][j as usize];
            }
        }
        return merged;
    }

    pub fn next_move_tremaux(&mut self) -> Option<(RelativeDirection, CardinalDirection)> {
        let moves: [(CardinalDirection, (isize, isize)); 4] = [
            (CardinalDirection::North, (-2, 0)),
            (CardinalDirection::East, (0, 2)),
            (CardinalDirection::South, (2, 0)),
            (CardinalDirection::West, (0, -2)),
        ];

        let (player_row, player_column) = self.player_position;
        let mut best: Option<(CardinalDirection, (isize, isize), u32)> = None;
        for (dir, (row_offset, column_offset)) in moves.iter() {
            let new_player_row: isize = player_row + row_offset;
            let new_player_column: isize = player_column + column_offset;
            if new_player_row < 0
                || new_player_column < 0
                || self.grid.len() as isize <= new_player_row
                || self.grid[0].len() as isize <= new_player_column
            {
                continue;
            }
            let cell: &String = &self.grid[new_player_row as usize][new_player_column as usize];
            if cell == "•" || cell == "-" || cell == "|" {
                continue;
            }
            let visits: u32 = self.visits[new_player_row as usize][new_player_column as usize];
            if best.is_none() || visits < best.as_ref().unwrap().2 {
                best = Some((dir.clone(), (*row_offset, *column_offset), visits));
            }
        }

        if let Some((chosen_cardinal_direction, (row_offset, column_offset), _)) = best {
            let new_r: isize = player_row + row_offset;
            let new_c: isize = player_column + column_offset;
            self.player_position = (new_r, new_c);
            self.visits[new_r as usize][new_c as usize] += 1;
            let relative_direction: RelativeDirection = absolute_to_relative_direction(
                &self.current_cardinal_direction,
                &chosen_cardinal_direction,
            );
            self.current_cardinal_direction = chosen_cardinal_direction;

            return Some((relative_direction, chosen_cardinal_direction));
        } else {
            return None;
        }
    }
}

fn absolute_to_relative_direction(
    player_orientation: &CardinalDirection,
    target_direction: &CardinalDirection,
) -> RelativeDirection {
    match (player_orientation, target_direction) {
        (CardinalDirection::North, CardinalDirection::North)
        | (CardinalDirection::East, CardinalDirection::East)
        | (CardinalDirection::South, CardinalDirection::South)
        | (CardinalDirection::West, CardinalDirection::West) => RelativeDirection::Front,

        (CardinalDirection::North, CardinalDirection::East)
        | (CardinalDirection::East, CardinalDirection::South)
        | (CardinalDirection::South, CardinalDirection::West)
        | (CardinalDirection::West, CardinalDirection::North) => RelativeDirection::Right,

        (CardinalDirection::North, CardinalDirection::West)
        | (CardinalDirection::West, CardinalDirection::South)
        | (CardinalDirection::South, CardinalDirection::East)
        | (CardinalDirection::East, CardinalDirection::North) => RelativeDirection::Left,

        (CardinalDirection::North, CardinalDirection::South)
        | (CardinalDirection::South, CardinalDirection::North)
        | (CardinalDirection::East, CardinalDirection::West)
        | (CardinalDirection::West, CardinalDirection::East) => RelativeDirection::Back,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::radar::RadarView;
    use shared::utils::{print_string_matrix, string_to_strings};

    #[test]
    fn test_merge_radar_views_with_directions_1() {
        // Spawns.
        let radar_1: Vec<Vec<String>> = vec![
            string_to_strings("#######"),
            string_to_strings("#######"),
            string_to_strings("•-•-•-•"),
            string_to_strings("   P   "),
            string_to_strings("•-• •-•"),
            string_to_strings("##|  A "),
            string_to_strings("##•-•-•"),
        ];

        print_string_matrix("radar view 1", &radar_1);
        let mut map: Map = Map::new(&radar_1, CardinalDirection::North);
        print_string_matrix("map + radar view 1", &map.grid);
        assert_eq!(map.player_position, (3, 3));
        assert_eq!(radar_1, map.grid);

        // Moves West.
        let radar_2: Vec<Vec<String>> = vec![
            string_to_strings("#######"),
            string_to_strings("#######"),
            string_to_strings("•-•-•-•"),
            string_to_strings("   P   "),
            string_to_strings("• •-• •"),
            string_to_strings("| ###  "),
            string_to_strings("• ###-•"),
        ];
        let expected_grid_2: Vec<Vec<String>> = vec![
            string_to_strings("#########"),
            string_to_strings("#########"),
            string_to_strings("•-•-•-•-•"),
            string_to_strings("         "),
            string_to_strings("• •-• •-•"),
            string_to_strings("| ##|  A "),
            string_to_strings("• ##•-•-•"),
        ];
        print_string_matrix("radar view 2", &radar_2);
        map.merge_radar_view(&radar_2, CardinalDirection::West);
        print_string_matrix("map + radar view 2", &map.grid);
        assert_eq!(map.player_position, (3, 3));
        assert_eq!(map.grid, expected_grid_2);

        // Moves East.
        let radar_3: Vec<Vec<String>> = vec![
            string_to_strings("##### •"),
            string_to_strings("##### |"),
            string_to_strings("•-•-• •"),
            string_to_strings("   P  |"),
            string_to_strings("• •-• •"),
            string_to_strings("|  A   "),
            string_to_strings("•-•-•-•"),
        ];
        print_string_matrix("radar view 3", &radar_3);
        map.merge_radar_view(&radar_3, CardinalDirection::East);
        print_string_matrix("map + radar view 3", &map.grid);
        assert_eq!(map.player_position, (3, 5));

        // Moves East.
        let radar_4: Vec<Vec<String>> = vec![
            string_to_strings("##• •##"),
            string_to_strings("##| |##"),
            string_to_strings("•-• •##"),
            string_to_strings("   P|##"),
            string_to_strings("•-• •##"),
            string_to_strings(" A   ##"),
            string_to_strings("•-•-•-•"),
        ];
        print_string_matrix("radar view 4", &radar_4);
        map.merge_radar_view(&radar_4, CardinalDirection::East);
        print_string_matrix("map + radar view 4", &map.grid);
        assert_eq!(map.player_position, (3, 7));

        // Moves South.
        let radar_5: Vec<Vec<String>> = vec![
            string_to_strings("•-• •##"),
            string_to_strings("    |##"),
            string_to_strings("•-• •-•"),
            string_to_strings(" A P   "),
            string_to_strings("•-•-•-•"),
            string_to_strings("#######"),
            string_to_strings("#######"),
        ];
        print_string_matrix("radar view 5", &radar_5);
        map.merge_radar_view(&radar_5, CardinalDirection::South);
        print_string_matrix("map + radar view 5", &map.grid);
        assert_eq!(map.player_position, (5, 7));

        // Moves North.
        let radar_6: Vec<Vec<String>> = vec![
            string_to_strings("##• •##"),
            string_to_strings("##| |##"),
            string_to_strings("•-• •##"),
            string_to_strings("   P|##"),
            string_to_strings("•-• •##"),
            string_to_strings(" A   ##"),
            string_to_strings("•-•-•-•"),
        ];
        print_string_matrix("radar view 6", &radar_6);
        map.merge_radar_view(&radar_6, CardinalDirection::North);
        print_string_matrix("map + radar view 6", &map.grid);
        assert_eq!(map.player_position, (3, 7));

        // Moves North.
        let radar_7: Vec<Vec<String>> = vec![
            string_to_strings("•-• •-•"),
            string_to_strings("|    A|"),
            string_to_strings("##• •##"),
            string_to_strings("##|P|##"),
            string_to_strings("•-• •##"),
            string_to_strings("    |##"),
            string_to_strings("•-• •##"),
        ];
        print_string_matrix("radar view 7", &radar_7);
        map.merge_radar_view(&radar_7, CardinalDirection::North);
        print_string_matrix("map + radar view 7", &map.grid);
        assert_eq!(map.player_position, (3, 7));

        // Moves South.
        let radar_8: Vec<Vec<String>> = vec![
            string_to_strings("##• •##"),
            string_to_strings("##| |##"),
            string_to_strings("•-• •##"),
            string_to_strings("   P|##"),
            string_to_strings("•-• •##"),
            string_to_strings(" A   ##"),
            string_to_strings("•-•-•-•"),
        ];
        print_string_matrix("radar view 8", &radar_8);
        map.merge_radar_view(&radar_8, CardinalDirection::South);
        print_string_matrix("map + radar view 8", &map.grid);
        assert_eq!(map.player_position, (5, 7));

        let expected_final_grid: Vec<Vec<String>> = vec![
            string_to_strings("####•-• •-•"),
            string_to_strings("####|    A|"),
            string_to_strings("######• •##"),
            string_to_strings("######| |##"),
            string_to_strings("•-•-•-• •##"),
            string_to_strings("        |##"),
            string_to_strings("• • •-• •-•"),
            string_to_strings("| |  A     "),
            string_to_strings("• •-•-•-•-•"),
            string_to_strings("###########"),
            string_to_strings("###########"),
        ];

        print_string_matrix("expected final grid", &expected_final_grid);

        assert_eq!(map.grid, expected_final_grid);
    }

    #[test]
    fn test_merge_radar_views_with_directions_2() {
        let radar_view_1: RadarView =
            RadarView::new(String::from("bKLzjzIMaaap8aa"), CardinalDirection::North);

        let radar_1: Vec<Vec<String>> = vec![
            string_to_strings("• • •-•"),
            string_to_strings("| |   |"),
            string_to_strings("• • •##"),
            string_to_strings("|   |##"),
            string_to_strings("•-• •##"),
            string_to_strings("##|    "),
            string_to_strings("##• •-•"),
        ];

        print_string_matrix("radar view 1", radar_view_1.grid.as_ref());
        let mut map: Map = Map::new(&radar_view_1.grid, CardinalDirection::North);
        print_string_matrix("map + radar view 1", &map.grid);
        assert_eq!(map.player_position, (3, 3));
        assert_eq!(radar_1, radar_view_1.grid);

        let radar_2: Vec<Vec<String>> = vec![
            string_to_strings("• •-•##"),
            string_to_strings("|   |##"),
            string_to_strings("##• •-•"),
            string_to_strings("##|   |"),
            string_to_strings("##• • •"),
            string_to_strings("|   |  "),
            string_to_strings("•-• • •"),
        ];
        let expected_grid_2: Vec<Vec<String>> = vec![
            string_to_strings("• •-•##"),
            string_to_strings("|   |##"),
            string_to_strings("• • •-•"),
            string_to_strings("| |   |"),
            string_to_strings("• • • •"),
            string_to_strings("|   |  "),
            string_to_strings("•-• • •"),
            string_to_strings("##|    "),
            string_to_strings("##• •-•"),
        ];

        print_string_matrix("radar view 2", &radar_2);
        let radar_view_2: RadarView =
            RadarView::new(String::from("zwfGMsAyap8aaaa"), CardinalDirection::North);
        assert_eq!(&radar_view_2.grid, &radar_2);
        let map_2: Map = Map::new(&radar_view_2.grid, CardinalDirection::North);
        assert_eq!(map_2.grid, radar_view_2.grid);
        map.merge_radar_view(&radar_view_2.grid, CardinalDirection::North);
        print_string_matrix("map + radar view 2", &map.grid);
        print_string_matrix("map + radar view 2 expected", &expected_grid_2);
        assert_eq!(map.player_position, (3, 3));
        // assert_eq!(map.grid, radar_view_2.grid);
        assert_eq!(map.grid, expected_grid_2);
    }

    #[test]
    fn test_tremaux_algorithm() {
        let grid: Vec<Vec<String>> = vec![
            string_to_strings("•-•-•"),
            string_to_strings("| | |"),
            string_to_strings("•-•-•"),
            string_to_strings("| | |"),
            string_to_strings("•-•-•"),
        ];
        let mut map: Map = Map::new(&grid, CardinalDirection::North);
        map.player_position = (2, 2);
        map.current_cardinal_direction = CardinalDirection::North;

        assert_eq!(map.next_move_tremaux(), Option::None);

        map.grid[2][4] = String::from(" ");
        map.visits[2][4] = 0;

        match map.next_move_tremaux() {
            Some((relative_direction, chosen_cardinal_direction)) => {
                assert_eq!(relative_direction, RelativeDirection::Right);
                assert_eq!(chosen_cardinal_direction, CardinalDirection::East);
                assert_eq!(map.player_position, (2, 4));
            }
            None => panic!("Expected a move."),
        }
    }
}
