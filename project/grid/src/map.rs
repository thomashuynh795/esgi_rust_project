use shared::log_debug;
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

    pub fn merge_radar_view(
        &mut self,
        new_view: &Vec<Vec<String>>,
        move_direction: CardinalDirection,
    ) -> () {
        // log_debug!(
        //     "merge_radar_view >>> Cardinal direction: {:?}",
        //     move_direction
        // );
        self.current_cardinal_direction = move_direction;
        // print_string_matrix("merge_radar_view >>> Map before update", &self.grid);
        // log_debug!("merge_radar_view >>> Rows number: {}", self.grid.len());
        // log_debug!(
        //     "merge_radar_view >>> Columns number: {}",
        //     self.grid[0].len()
        // );
        self.expand_grid_if_needed();
        // print_string_matrix("merge_radar_view >>> Map after expansion", &self.grid);
        // log_debug!("merge_radar_view >>> Rows number: {}", self.grid.len());
        // log_debug!(
        //     "merge_radar_view >>> Columns number: {}",
        //     self.grid[0].len()
        // );
        self.merge_radar_view_to_map_grid(new_view);
    }

    pub fn merge_radar_view_to_map_grid(&mut self, new_view: &Vec<Vec<String>>) {
        let view_size: usize = 7;
        let player_row: usize = self.player_position.0 as usize;
        let player_col: usize = self.player_position.1 as usize;

        let mut lowest_row: usize = if player_row >= 3 { player_row - 3 } else { 0 };
        let mut lowest_col: usize = if player_col >= 3 { player_col - 3 } else { 0 };

        if self.grid.len() >= view_size {
            lowest_row = lowest_row.min(self.grid.len() - view_size);
        }
        if self.grid[0].len() >= view_size {
            lowest_col = lowest_col.min(self.grid[0].len() - view_size);
        }

        log_debug!(
            "grid rows: {} cols: {}",
            self.grid.len(),
            self.grid[0].len()
        );
        for i in 0..view_size {
            for j in 0..view_size {
                log_debug!("merging cell ({}, {})", i, j);
                self.grid[lowest_row + i][lowest_col + j] = Map::select_string_to_save(
                    &self.grid[lowest_row + i][lowest_col + j],
                    &new_view[i][j],
                )
                .clone();
            }
        }
    }

    pub fn select_string_to_save<'a>(
        grid_string: &'a String,
        radar_view_string: &'a String,
    ) -> &'a String {
        if radar_view_string != "#" {
            radar_view_string
        } else {
            grid_string
        }
    }

    pub fn should_expand_grid(&self, next_cardinal_direction: CardinalDirection) -> bool {
        let (row_offset, col_offset) = match next_cardinal_direction {
            CardinalDirection::North => (-2, 0),
            CardinalDirection::South => (2, 0),
            CardinalDirection::East => (0, 2),
            CardinalDirection::West => (0, -2),
        };

        let new_player_pos: (isize, isize) = (
            self.player_position.0 + row_offset,
            self.player_position.1 + col_offset,
        );

        let grid_rows: isize = self.grid.len() as isize;
        let grid_cols: isize = if grid_rows == 0 {
            0
        } else {
            self.grid[0].len() as isize
        };

        match next_cardinal_direction {
            CardinalDirection::North => new_player_pos.0 - 3 < 0,
            CardinalDirection::South => new_player_pos.0 + 3 >= grid_rows,
            CardinalDirection::East => new_player_pos.1 + 3 >= grid_cols,
            CardinalDirection::West => new_player_pos.1 - 3 < 0,
        }
    }

    pub fn expand_grid_if_needed(&mut self) {
        let grid_rows = self.grid.len() as isize;
        let grid_cols = if grid_rows == 0 {
            0
        } else {
            self.grid[0].len() as isize
        };

        let mut expand_top: isize = 0;
        let mut expand_left: isize = 0;
        let mut expand_bottom: isize = 0;
        let mut expand_right: isize = 0;

        if self.player_position.0 - 3 < 0 {
            expand_top = 2;
        }
        if self.player_position.0 + 3 >= grid_rows {
            expand_bottom = 2;
        }
        if self.player_position.1 - 3 < 0 {
            expand_left = 2;
        }
        if self.player_position.1 + 3 >= grid_cols {
            expand_right = 2;
        }

        let new_rows: isize = grid_rows + expand_top + expand_bottom;
        let new_cols: isize = grid_cols + expand_left + expand_right;

        if new_rows == grid_rows && new_cols == grid_cols {
            return;
        }

        let mut new_grid: Vec<Vec<String>> =
            vec![vec![String::from("#"); new_cols as usize]; new_rows as usize];
        for i in 0..grid_rows {
            for j in 0..grid_cols {
                new_grid[(i + expand_top) as usize][(j + expand_left) as usize] =
                    self.grid[i as usize][j as usize].clone();
            }
        }

        self.player_position.0 += expand_top;
        self.player_position.1 += expand_left;

        let old_visit_rows = self.visits.len() as isize;
        let old_visit_cols = if old_visit_rows == 0 {
            0
        } else {
            self.visits[0].len() as isize
        };
        let mut new_visits: Vec<Vec<u32>> = vec![vec![0; new_cols as usize]; new_rows as usize];
        for i in 0..old_visit_rows {
            for j in 0..old_visit_cols {
                new_visits[(i + expand_top) as usize][(j + expand_left) as usize] =
                    self.visits[i as usize][j as usize];
            }
        }
        self.visits = new_visits;
        self.grid = new_grid;
    }

    pub fn next_move_tremaux(&mut self) -> Option<(RelativeDirection, CardinalDirection)> {
        let moves: [(CardinalDirection, (isize, isize), (isize, isize)); 4] = [
            (CardinalDirection::North, (-2, 0), (-1, 0)),
            (CardinalDirection::East, (0, 2), (0, 1)),
            (CardinalDirection::South, (2, 0), (1, 0)),
            (CardinalDirection::West, (0, -2), (0, -1)),
        ];

        let (player_row, player_column) = self.player_position;
        let mut best_move: Option<(CardinalDirection, (isize, isize), u32)> = None;

        for (dir, (row_offset, column_offset), (wall_row_offset, wall_col_offset)) in moves.iter() {
            let new_player_row: isize = player_row + row_offset;
            let new_player_column: isize = player_column + column_offset;
            let wall_row: isize = player_row + wall_row_offset;
            let wall_col: isize = player_column + wall_col_offset;

            if new_player_row < 0
                || new_player_column < 0
                || self.grid.len() as isize <= new_player_row
                || self.grid[0].len() as isize <= new_player_column
            {
                continue;
            }

            let wall: &String = &self.grid[wall_row as usize][wall_col as usize];
            if wall == "-" || wall == "|" {
                continue;
            }

            let cell = &self.grid[new_player_row as usize][new_player_column as usize];

            if cell == "•" || cell == "-" || cell == "|" {
                continue;
            }

            let visits = self.visits[new_player_row as usize][new_player_column as usize];

            if best_move.is_none() || visits < best_move.as_ref().unwrap().2 {
                best_move = Some((dir.clone(), (*row_offset, *column_offset), visits));
            }
        }

        if let Some((chosen_cardinal_direction, (row_offset, column_offset), _)) = best_move {
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
    use shared::utils::{print_string_matrix, string_to_strings};

    // #[test]
    // fn test_update_player_position_in_new_grid() {
    //     let grid_1: Vec<Vec<String>> = vec![
    //         string_to_strings("•-•-•-•"),
    //         string_to_strings("|1|2|3|"),
    //         string_to_strings("•-•-•-•"),
    //         string_to_strings("|4|5|6|"),
    //         string_to_strings("•-•-•-•"),
    //         string_to_strings("|7|8|9|"),
    //         string_to_strings("•-•-•-•"),
    //     ];
    //     let mut map_1: Map = Map::new(&grid_1, CardinalDirection::North);
    //     map_1.player_position = (3, 3);
    //     // log_debug!("Player position before: {:?}", map_1.player_position);
    //     map_1.update_player_position_in_new_grid();
    //     // log_debug!("Player position after: {:?}", map_1.player_position);
    //     assert_eq!(map_1.player_position, (3, 3));
    //     map_1.player_position = (3, 3);
    //     map_1.current_cardinal_direction = CardinalDirection::East;
    //     map_1.update_player_position_in_new_grid();
    //     assert_eq!(map_1.player_position, (3, 5));
    //     map_1.player_position = (3, 3);
    //     map_1.current_cardinal_direction = CardinalDirection::South;
    //     map_1.update_player_position_in_new_grid();
    //     assert_eq!(map_1.player_position, (5, 3));
    //     map_1.player_position = (3, 3);
    //     map_1.current_cardinal_direction = CardinalDirection::West;
    //     map_1.update_player_position_in_new_grid();
    //     assert_eq!(map_1.player_position, (3, 3));

    //     let grid_2: Vec<Vec<String>> = vec![
    //         string_to_strings("•-•-•-•-•-•-•-•"),
    //         string_to_strings("|1|2|3|4|5|6|7|"),
    //         string_to_strings("•-•-•-•-•-•-•-•"),
    //         string_to_strings("|1|2|3|4|5|6|7|"),
    //         string_to_strings("•-•-•-•-•-•-•-•"),
    //         string_to_strings("|1|2|3|4|5|6|7|"),
    //         string_to_strings("•-•-•-•-•-•-•-•"),
    //         string_to_strings("|1|2|3|4|5|6|7|"),
    //         string_to_strings("•-•-•-•-•-•-•-•"),
    //         string_to_strings("|1|2|3|4|5|6|7|"),
    //         string_to_strings("•-•-•-•-•-•-•-•"),
    //         string_to_strings("|1|2|3|4|5|6|7|"),
    //         string_to_strings("•-•-•-•-•-•-•-•"),
    //         string_to_strings("|1|2|3|4|5|6|7|"),
    //         string_to_strings("•-•-•-•-•-•-•-•"),
    //     ];

    //     log_debug!("OTHER OTHER OTHER");

    //     let mut map_2: Map = Map::new(&grid_2, CardinalDirection::North);
    //     map_2.player_position = (7, 7);
    //     log_debug!("Player position before: {:?}", map_2.player_position);
    //     map_2.update_player_position_in_new_grid();
    //     log_debug!("Player position after: {:?}", map_2.player_position);
    //     assert_eq!(map_2.player_position, (5, 7));
    //     map_2.player_position = (7, 7);
    //     map_2.current_cardinal_direction = CardinalDirection::East;
    //     map_2.update_player_position_in_new_grid();
    //     assert_eq!(map_2.player_position, (7, 9));
    //     map_2.player_position = (7, 7);
    //     map_2.current_cardinal_direction = CardinalDirection::South;
    //     map_2.update_player_position_in_new_grid();
    //     assert_eq!(map_2.player_position, (9, 7));
    //     map_2.player_position = (7, 7);
    //     map_2.current_cardinal_direction = CardinalDirection::West;
    //     map_2.update_player_position_in_new_grid();
    //     assert_eq!(map_2.player_position, (7, 5));
    // }

    #[test]
    fn test_should_expand_grid() {
        let grid_1: Vec<Vec<String>> = vec![
            string_to_strings("•-•-•-•"),
            string_to_strings("|1|2|3|"),
            string_to_strings("•-•-•-•"),
            string_to_strings("|4|5|6|"),
            string_to_strings("•-•-•-•"),
            string_to_strings("|7|8|9|"),
            string_to_strings("•-•-•-•"),
        ];
        let mut map_1: Map = Map::new(&grid_1, CardinalDirection::North);
        map_1.player_position = (3, 3);
        assert_eq!(map_1.should_expand_grid(CardinalDirection::North), true);
        assert_eq!(map_1.should_expand_grid(CardinalDirection::East), true);
        assert_eq!(map_1.should_expand_grid(CardinalDirection::South), true);
        assert_eq!(map_1.should_expand_grid(CardinalDirection::West), true);

        let grid_2: Vec<Vec<String>> = vec![
            string_to_strings("•-•-•-•-•-•-•-•"),
            string_to_strings("|1|2|3|4|5|6|7|"),
            string_to_strings("•-•-•-•-•-•-•-•"),
            string_to_strings("|1|2|3|4|5|6|7|"),
            string_to_strings("•-•-•-•-•-•-•-•"),
            string_to_strings("|1|2|3|4|5|6|7|"),
            string_to_strings("•-•-•-•-•-•-•-•"),
            string_to_strings("|1|2|3|4|5|6|7|"),
            string_to_strings("•-•-•-•-•-•-•-•"),
            string_to_strings("|1|2|3|4|5|6|7|"),
            string_to_strings("•-•-•-•-•-•-•-•"),
            string_to_strings("|1|2|3|4|5|6|7|"),
            string_to_strings("•-•-•-•-•-•-•-•"),
            string_to_strings("|1|2|3|4|5|6|7|"),
            string_to_strings("•-•-•-•-•-•-•-•"),
        ];
        let mut map_2: Map = Map::new(&grid_2, CardinalDirection::North);
        map_2.player_position = (7, 7);
        assert_eq!(map_2.should_expand_grid(CardinalDirection::North), false);
        assert_eq!(map_2.should_expand_grid(CardinalDirection::East), false);
        assert_eq!(map_2.should_expand_grid(CardinalDirection::South), false);
        assert_eq!(map_2.should_expand_grid(CardinalDirection::West), false);
        map_2.player_position = (1, 1);
        assert_eq!(map_2.should_expand_grid(CardinalDirection::North), true);
        assert_eq!(map_2.should_expand_grid(CardinalDirection::East), false);
        assert_eq!(map_2.should_expand_grid(CardinalDirection::South), false);
        assert_eq!(map_2.should_expand_grid(CardinalDirection::West), true);
        map_2.player_position = (1, 13);
        assert_eq!(map_2.should_expand_grid(CardinalDirection::North), true);
        assert_eq!(map_2.should_expand_grid(CardinalDirection::East), true);
        assert_eq!(map_2.should_expand_grid(CardinalDirection::South), false);
        assert_eq!(map_2.should_expand_grid(CardinalDirection::West), false);
        map_2.player_position = (13, 1);
        assert_eq!(map_2.should_expand_grid(CardinalDirection::North), false);
        assert_eq!(map_2.should_expand_grid(CardinalDirection::East), false);
        assert_eq!(map_2.should_expand_grid(CardinalDirection::South), true);
        assert_eq!(map_2.should_expand_grid(CardinalDirection::West), true);
        map_2.player_position = (13, 13);
        assert_eq!(map_2.should_expand_grid(CardinalDirection::North), false);
        assert_eq!(map_2.should_expand_grid(CardinalDirection::East), true);
        assert_eq!(map_2.should_expand_grid(CardinalDirection::South), true);
        assert_eq!(map_2.should_expand_grid(CardinalDirection::West), false);
        map_2.player_position = (3, 3);
        assert_eq!(map_2.should_expand_grid(CardinalDirection::North), true);
        assert_eq!(map_2.should_expand_grid(CardinalDirection::East), false);
        assert_eq!(map_2.should_expand_grid(CardinalDirection::South), false);
        assert_eq!(map_2.should_expand_grid(CardinalDirection::West), true);
        map_2.player_position = (3, 11);
        assert_eq!(map_2.should_expand_grid(CardinalDirection::North), true);
        assert_eq!(map_2.should_expand_grid(CardinalDirection::East), true);
        assert_eq!(map_2.should_expand_grid(CardinalDirection::South), false);
        assert_eq!(map_2.should_expand_grid(CardinalDirection::West), false);
        map_2.player_position = (11, 3);
        assert_eq!(map_2.should_expand_grid(CardinalDirection::North), false);
        assert_eq!(map_2.should_expand_grid(CardinalDirection::East), false);
        assert_eq!(map_2.should_expand_grid(CardinalDirection::South), true);
        assert_eq!(map_2.should_expand_grid(CardinalDirection::West), true);
        map_2.player_position = (11, 11);
        assert_eq!(map_2.should_expand_grid(CardinalDirection::North), false);
        assert_eq!(map_2.should_expand_grid(CardinalDirection::East), true);
        assert_eq!(map_2.should_expand_grid(CardinalDirection::South), true);
        assert_eq!(map_2.should_expand_grid(CardinalDirection::West), false);
    }

    #[test]
    fn test_expand_grid_if_needed() {
        let grid_1: Vec<Vec<String>> = vec![
            string_to_strings("•-•-•-•-•-•-•-•"),
            string_to_strings("|1|2|3|4|5|6|7|"),
            string_to_strings("•-•-•-•-•-•-•-•"),
            string_to_strings("|1|2|3|4|5|6|7|"),
            string_to_strings("•-•-•-•-•-•-•-•"),
            string_to_strings("|1|2|3|4|5|6|7|"),
            string_to_strings("•-•-•-•-•-•-•-•"),
            string_to_strings("|1|2|3|4|5|6|7|"),
            string_to_strings("•-•-•-•-•-•-•-•"),
            string_to_strings("|1|2|3|4|5|6|7|"),
            string_to_strings("•-•-•-•-•-•-•-•"),
            string_to_strings("|1|2|3|4|5|6|7|"),
            string_to_strings("•-•-•-•-•-•-•-•"),
            string_to_strings("|1|2|3|4|5|6|7|"),
            string_to_strings("•-•-•-•-•-•-•-•"),
        ];

        let north_grid: Vec<Vec<String>> = vec![
            string_to_strings("•#•#•#•#•#•#•#•"),
            string_to_strings("###############"),
            string_to_strings("•-•-•-•-•-•-•-•"),
            string_to_strings("|1|2|3|4|5|6|7|"),
            string_to_strings("•-•-•-•-•-•-•-•"),
            string_to_strings("|1|2|3|4|5|6|7|"),
            string_to_strings("•-•-•-•-•-•-•-•"),
            string_to_strings("|1|2|3|4|5|6|7|"),
            string_to_strings("•-•-•-•-•-•-•-•"),
            string_to_strings("|1|2|3|4|5|6|7|"),
            string_to_strings("•-•-•-•-•-•-•-•"),
            string_to_strings("|1|2|3|4|5|6|7|"),
            string_to_strings("•-•-•-•-•-•-•-•"),
            string_to_strings("|1|2|3|4|5|6|7|"),
            string_to_strings("•-•-•-•-•-•-•-•"),
            string_to_strings("|1|2|3|4|5|6|7|"),
            string_to_strings("•-•-•-•-•-•-•-•"),
        ];

        let east_grid: Vec<Vec<String>> = vec![
            string_to_strings("•-•-•-•-•-•-•-•#•"),
            string_to_strings("|1|2|3|4|5|6|7|##"),
            string_to_strings("•-•-•-•-•-•-•-•#•"),
            string_to_strings("|1|2|3|4|5|6|7|##"),
            string_to_strings("•-•-•-•-•-•-•-•#•"),
            string_to_strings("|1|2|3|4|5|6|7|##"),
            string_to_strings("•-•-•-•-•-•-•-•#•"),
            string_to_strings("|1|2|3|4|5|6|7|##"),
            string_to_strings("•-•-•-•-•-•-•-•#•"),
            string_to_strings("|1|2|3|4|5|6|7|##"),
            string_to_strings("•-•-•-•-•-•-•-•#•"),
            string_to_strings("|1|2|3|4|5|6|7|##"),
            string_to_strings("•-•-•-•-•-•-•-•#•"),
            string_to_strings("|1|2|3|4|5|6|7|##"),
            string_to_strings("•-•-•-•-•-•-•-•#•"),
        ];

        let south_grid: Vec<Vec<String>> = vec![
            string_to_strings("•-•-•-•-•-•-•-•"),
            string_to_strings("|1|2|3|4|5|6|7|"),
            string_to_strings("•-•-•-•-•-•-•-•"),
            string_to_strings("|1|2|3|4|5|6|7|"),
            string_to_strings("•-•-•-•-•-•-•-•"),
            string_to_strings("|1|2|3|4|5|6|7|"),
            string_to_strings("•-•-•-•-•-•-•-•"),
            string_to_strings("|1|2|3|4|5|6|7|"),
            string_to_strings("•-•-•-•-•-•-•-•"),
            string_to_strings("|1|2|3|4|5|6|7|"),
            string_to_strings("•-•-•-•-•-•-•-•"),
            string_to_strings("|1|2|3|4|5|6|7|"),
            string_to_strings("•-•-•-•-•-•-•-•"),
            string_to_strings("|1|2|3|4|5|6|7|"),
            string_to_strings("•-•-•-•-•-•-•-•"),
            string_to_strings("###############"),
            string_to_strings("•#•#•#•#•#•#•#•"),
        ];

        let west_grid: Vec<Vec<String>> = vec![
            string_to_strings("•#•-•-•-•-•-•-•-•"),
            string_to_strings("##|1|2|3|4|5|6|7|"),
            string_to_strings("•#•-•-•-•-•-•-•-•"),
            string_to_strings("##|1|2|3|4|5|6|7|"),
            string_to_strings("•#•-•-•-•-•-•-•-•"),
            string_to_strings("##|1|2|3|4|5|6|7|"),
            string_to_strings("•#•-•-•-•-•-•-•-•"),
            string_to_strings("##|1|2|3|4|5|6|7|"),
            string_to_strings("•#•-•-•-•-•-•-•-•"),
            string_to_strings("##|1|2|3|4|5|6|7|"),
            string_to_strings("•#•-•-•-•-•-•-•-•"),
            string_to_strings("##|1|2|3|4|5|6|7|"),
            string_to_strings("•#•-•-•-•-•-•-•-•"),
            string_to_strings("##|1|2|3|4|5|6|7|"),
            string_to_strings("•#•-•-•-•-•-•-•-•"),
        ];
        let mut map_1: Map = Map::new(&grid_1, CardinalDirection::North);
        let mut map_2: Map = Map::new(&grid_1, CardinalDirection::East);
        let mut map_3: Map = Map::new(&grid_1, CardinalDirection::South);
        let mut map_4: Map = Map::new(&grid_1, CardinalDirection::West);

        map_1.player_position = (3, 3);
        map_2.player_position = (12, 12);
        map_3.player_position = (12, 12);
        map_4.player_position = (3, 3);

        assert_eq!(map_1.grid.len(), 15);
        assert_eq!(map_2.grid.len(), 15);
        assert_eq!(map_3.grid.len(), 15);
        assert_eq!(map_4.grid.len(), 15);

        assert_eq!(map_1.grid[0].len(), 15);
        assert_eq!(map_2.grid[0].len(), 15);
        assert_eq!(map_3.grid[0].len(), 15);
        assert_eq!(map_4.grid[0].len(), 15);

        map_1.expand_grid_if_needed();
        map_2.expand_grid_if_needed();
        map_3.expand_grid_if_needed();
        map_4.expand_grid_if_needed();

        print_string_matrix("north grid", &north_grid);
        log_debug!("map_1.grid.len() = {}", map_1.grid.len());

        assert_eq!(map_1.grid.len(), 17);
        assert_eq!(map_2.grid.len(), 15);
        assert_eq!(map_3.grid.len(), 17);
        assert_eq!(map_4.grid.len(), 15);

        assert_eq!(map_1.grid[0].len(), 15);
        assert_eq!(map_2.grid[0].len(), 17);
        assert_eq!(map_3.grid[0].len(), 15);
        assert_eq!(map_4.grid[0].len(), 17);

        assert_eq!(map_1.grid, north_grid);
        assert_eq!(map_2.grid, east_grid);
        assert_eq!(map_3.grid, south_grid);
        assert_eq!(map_4.grid, west_grid);
    }

    // #[test]
    // fn test_update_player_position_in_new_grid_1() {
    //     let grid: Vec<Vec<String>> = vec![
    //         string_to_strings("•-•-•-•"),
    //         string_to_strings("|1|2|3|"),
    //         string_to_strings("•-•-•-•"),
    //         string_to_strings("|4|5|6|"),
    //         string_to_strings("•-•-•-•"),
    //         string_to_strings("|7|8|9|"),
    //         string_to_strings("•-•-•-•"),
    //     ];
    //     let mut map: Map = Map::new(&grid, CardinalDirection::North);
    //     map.player_position = (3, 3);
    //     map.current_cardinal_direction = CardinalDirection::North;

    //     assert_eq!(map.player_position, (3, 3));
    //     assert_eq!(map.current_cardinal_direction, CardinalDirection::North);

    //     map.update_player_position_in_new_grid();
    //     assert_eq!(map.player_position, (3, 3));
    //     assert_eq!(map.current_cardinal_direction, CardinalDirection::North);

    //     map.current_cardinal_direction = CardinalDirection::East;
    //     map.update_player_position_in_new_grid();
    //     assert_eq!(map.player_position, (3, 5));
    //     assert_eq!(map.current_cardinal_direction, CardinalDirection::East);

    //     map.current_cardinal_direction = CardinalDirection::South;
    //     map.update_player_position_in_new_grid();
    //     assert_eq!(map.player_position, (5, 5));
    //     assert_eq!(map.current_cardinal_direction, CardinalDirection::South);

    //     map.current_cardinal_direction = CardinalDirection::West;
    //     map.update_player_position_in_new_grid();
    //     assert_eq!(map.player_position, (5, 3));
    //     assert_eq!(map.current_cardinal_direction, CardinalDirection::West);
    // }

    #[test]
    fn test_merge_radar_views_with_directions_1() {
        // Spawns.
        let radar_1: Vec<Vec<String>> = vec![
            string_to_strings("#######"),
            string_to_strings("#######"),
            string_to_strings("•-•-•-•"),
            string_to_strings("       "),
            string_to_strings("•-• •-•"),
            string_to_strings("##|  A "),
            string_to_strings("##•-•-•"),
        ];

        print_string_matrix("radar view 1", &radar_1);
        let mut map: Map = Map::new(&radar_1, CardinalDirection::North);
        print_string_matrix("map + radar view 1", &map.grid);
        assert_eq!(map.player_position, (3, 3));
        assert_eq!(radar_1, map.grid);
        let map_with_radar_1_rows_number: usize = map.grid.len();
        let map_with_radar_1_columns_number: usize = if 0 < map_with_radar_1_rows_number {
            map.grid[0].len()
        } else {
            0
        };
        assert_eq!(map_with_radar_1_rows_number, 7);
        assert_eq!(map_with_radar_1_columns_number, 7);

        // Moves West.
        let radar_2: Vec<Vec<String>> = vec![
            string_to_strings("#######"),
            string_to_strings("#######"),
            string_to_strings("•-•-•-•"),
            string_to_strings("       "),
            string_to_strings("• •-• •"),
            string_to_strings("| ###  "),
            string_to_strings("• ###-•"),
        ];
        let expected_grid_2: Vec<Vec<String>> = vec![
            string_to_strings("•#•#•#•#•"),
            string_to_strings("#########"),
            string_to_strings("•-•-•-•-•"),
            string_to_strings("         "),
            string_to_strings("• •-• •-•"),
            string_to_strings("| ##|  A "),
            string_to_strings("•#•#•-•-•"),
        ];
        print_string_matrix("radar view 2", &radar_2);
        map.merge_radar_view(&radar_2, CardinalDirection::West);
        print_string_matrix("map + radar view 2", &map.grid);
        log_debug!("{:?}", map.player_position);
        assert_eq!(map.player_position, (3, 3));
        // print_string_matrix("map", matrix);
        assert_eq!(map.grid, expected_grid_2);
        let map_with_radar_2_rows_number: usize = map.grid.len();
        let map_with_radar_2_columns_number: usize = if 0 < map_with_radar_2_rows_number {
            map.grid[0].len()
        } else {
            0
        };
        assert_eq!(map_with_radar_2_rows_number, 7);
        assert_eq!(map_with_radar_2_columns_number, 9);

        // Moves East.
        let radar_3: Vec<Vec<String>> = vec![
            string_to_strings("##### •"),
            string_to_strings("##### |"),
            string_to_strings("•-•-• •"),
            string_to_strings("      |"),
            string_to_strings("• •-• •"),
            string_to_strings("|  A   "),
            string_to_strings("•-•-•-•"),
        ];
        print_string_matrix("radar view 3", &radar_3);
        map.merge_radar_view(&radar_3, CardinalDirection::East);
        print_string_matrix("map + radar view 3", &map.grid);
        log_debug!("{:?}", map.player_position);
        assert_eq!(map.player_position, (3, 5));
        let map_with_radar_3_rows_number: usize = map.grid.len();
        let map_with_radar_3_columns_number: usize = if 0 < map_with_radar_3_rows_number {
            map.grid[0].len()
        } else {
            0
        };
        assert_eq!(map_with_radar_3_rows_number, 7);
        assert_eq!(map_with_radar_3_columns_number, 9);

        // Moves East.
        let radar_4: Vec<Vec<String>> = vec![
            string_to_strings("##• •##"),
            string_to_strings("##| |##"),
            string_to_strings("•-• •##"),
            string_to_strings("    |##"),
            string_to_strings("•-• •##"),
            string_to_strings(" A   ##"),
            string_to_strings("•-•-•-•"),
        ];
        print_string_matrix("radar view 4", &radar_4);
        map.merge_radar_view(&radar_4, CardinalDirection::East);
        print_string_matrix("map + radar view 4", &map.grid);
        assert_eq!(map.player_position, (3, 7));
        let map_with_radar_4_rows_number: usize = map.grid.len();
        let map_with_radar_4_columns_number: usize = if 0 < map_with_radar_4_rows_number {
            map.grid[0].len()
        } else {
            0
        };
        assert_eq!(map_with_radar_4_rows_number, 7);
        assert_eq!(map_with_radar_4_columns_number, 11);

        // Moves South.
        let radar_5: Vec<Vec<String>> = vec![
            string_to_strings("•-• •##"),
            string_to_strings("    |##"),
            string_to_strings("•-• •-•"),
            string_to_strings(" A     "),
            string_to_strings("•-•-•-•"),
            string_to_strings("#######"),
            string_to_strings("#######"),
        ];
        print_string_matrix("radar view 5", &radar_5);
        map.merge_radar_view(&radar_5, CardinalDirection::South);
        print_string_matrix("map + radar view 5", &map.grid);
        assert_eq!(map.player_position, (5, 7));
        let map_with_radar_5_rows_number: usize = map.grid.len();
        let map_with_radar_5_columns_number: usize = if 0 < map_with_radar_5_rows_number {
            map.grid[0].len()
        } else {
            0
        };
        assert_eq!(map_with_radar_5_rows_number, 9);
        assert_eq!(map_with_radar_5_columns_number, 11);

        // Moves North.
        let radar_6: Vec<Vec<String>> = vec![
            string_to_strings("##• •##"),
            string_to_strings("##| |##"),
            string_to_strings("•-• •##"),
            string_to_strings("    |##"),
            string_to_strings("•-• •##"),
            string_to_strings(" A   ##"),
            string_to_strings("•-•-•-•"),
        ];
        print_string_matrix("radar view 6", &radar_6);
        map.merge_radar_view(&radar_6, CardinalDirection::North);
        print_string_matrix("map + radar view 6", &map.grid);
        assert_eq!(map.player_position, (3, 7));
        let map_with_radar_6_rows_number: usize = map.grid.len();
        let map_with_radar_6_columns_number: usize = if 0 < map_with_radar_6_rows_number {
            map.grid[0].len()
        } else {
            0
        };
        assert_eq!(map_with_radar_6_rows_number, 9);
        assert_eq!(map_with_radar_6_columns_number, 11);

        // Moves North.
        let radar_7: Vec<Vec<String>> = vec![
            string_to_strings("•-• •-•"),
            string_to_strings("|    A|"),
            string_to_strings("##• •##"),
            string_to_strings("##| |##"),
            string_to_strings("•-• •##"),
            string_to_strings("    |##"),
            string_to_strings("•-• •##"),
        ];
        print_string_matrix("radar view 7", &radar_7);
        map.merge_radar_view(&radar_7, CardinalDirection::North);
        print_string_matrix("map + radar view 7", &map.grid);
        assert_eq!(map.player_position, (3, 7));
        let map_with_radar_7_rows_number: usize = map.grid.len();
        let map_with_radar_7_columns_number: usize = if 0 < map_with_radar_7_rows_number {
            map.grid[0].len()
        } else {
            0
        };
        assert_eq!(map_with_radar_7_rows_number, 11);
        assert_eq!(map_with_radar_7_columns_number, 11);

        // Moves South.
        let radar_8: Vec<Vec<String>> = vec![
            string_to_strings("##• •##"),
            string_to_strings("##| |##"),
            string_to_strings("•-• •##"),
            string_to_strings("    |##"),
            string_to_strings("•-• •##"),
            string_to_strings(" A   ##"),
            string_to_strings("•-•-•-•"),
        ];
        print_string_matrix("radar view 8", &radar_8);
        map.merge_radar_view(&radar_8, CardinalDirection::South);
        print_string_matrix("map + radar view 8", &map.grid);
        assert_eq!(map.player_position, (5, 7));
        let map_with_radar_8_rows_number: usize = map.grid.len();
        let map_with_radar_8_columns_number: usize = if 0 < map_with_radar_8_rows_number {
            map.grid[0].len()
        } else {
            0
        };
        assert_eq!(map_with_radar_8_rows_number, 11);
        assert_eq!(map_with_radar_8_columns_number, 11);
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

    // #[test]
    // fn test_merge_radar_views_with_directions_2() {
    //     let radar_view_1: RadarView =
    //         RadarView::new(String::from("bKLzjzIMaaap8aa"), CardinalDirection::North);

    //     let radar_1: Vec<Vec<String>> = vec![
    //         string_to_strings("• • •-•"),
    //         string_to_strings("| |   |"),
    //         string_to_strings("• • •##"),
    //         string_to_strings("|   |##"),
    //         string_to_strings("•-• •##"),
    //         string_to_strings("##|    "),
    //         string_to_strings("##• •-•"),
    //     ];

    //     print_string_matrix("radar view 1", radar_view_1.grid.as_ref());
    //     let mut map: Map = Map::new(&radar_view_1.grid, CardinalDirection::North);
    //     print_string_matrix("map + radar view 1", &map.grid);
    //     assert_eq!(map.player_position, (3, 3));
    //     assert_eq!(radar_1, radar_view_1.grid);

    //     let radar_2: Vec<Vec<String>> = vec![
    //         string_to_strings("• •-•##"),
    //         string_to_strings("|   |##"),
    //         string_to_strings("##• •-•"),
    //         string_to_strings("##|   |"),
    //         string_to_strings("##• • •"),
    //         string_to_strings("|   |  "),
    //         string_to_strings("•-• • •"),
    //     ];
    //     let expected_grid_2: Vec<Vec<String>> = vec![
    //         string_to_strings("• •-•##"),
    //         string_to_strings("|   |##"),
    //         string_to_strings("• • •-•"),
    //         string_to_strings("| |   |"),
    //         string_to_strings("• • • •"),
    //         string_to_strings("|   |  "),
    //         string_to_strings("•-• • •"),
    //         string_to_strings("##|    "),
    //         string_to_strings("##• •-•"),
    //     ];

    //     print_string_matrix("radar view 2", &radar_2);
    //     let radar_view_2: RadarView =
    //         RadarView::new(String::from("zwfGMsAyap8aaaa"), CardinalDirection::North);
    //     assert_eq!(&radar_view_2.grid, &radar_2);
    //     let map_2: Map = Map::new(&radar_view_2.grid, CardinalDirection::North);
    //     assert_eq!(map_2.grid, radar_view_2.grid);
    //     map.merge_radar_view(&radar_view_2.grid, CardinalDirection::North);
    //     print_string_matrix("map + radar view 2", &map.grid);
    //     print_string_matrix("map + radar view 2 expected", &expected_grid_2);
    //     assert_eq!(map.player_position, (3, 3));
    //     // assert_eq!(map.grid, radar_view_2.grid);
    //     assert_eq!(map.grid, expected_grid_2);
    // }

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
