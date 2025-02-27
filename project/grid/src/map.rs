use shared::log_info;

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

pub struct Map {
    pub position: (isize, isize),
    pub grid: Vec<Vec<String>>,
}

impl Map {
    pub fn new(initial_view: &Vec<Vec<String>>) -> Map {
        let view_size = initial_view.len();
        let center = (
            view_size as isize / 2,
            if view_size > 0 {
                initial_view[0].len() as isize / 2
            } else {
                0
            },
        );
        log_info!("Player is spawning");
        let map = Map {
            position: center,
            grid: initial_view.clone(),
        };
        log_info!("Player has spawned");
        map
    }

    pub fn merge_radar_view(&mut self, new_view: &Vec<Vec<String>>, direction: Direction) {
        let (dr, dc) = match direction {
            Direction::North => (-2, 0),
            Direction::South => (2, 0),
            Direction::East => (0, 2),
            Direction::West => (0, -2),
        };

        let candidate = (self.position.0 + dr, self.position.1 + dc);

        let (new_grid, effective_position) =
            Map::merge_radar_view_to_map_grid(&self.grid, new_view, candidate);
        self.grid = new_grid;
        self.position = effective_position;
    }

    fn merge_radar_view_to_map_grid(
        saved: &Vec<Vec<String>>,
        new_view: &Vec<Vec<String>>,
        merge_center: (isize, isize),
    ) -> (Vec<Vec<String>>, (isize, isize)) {
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

        let new_rows = (overall_bottom - overall_top + 1) as usize;
        let new_cols = (overall_right - overall_left + 1) as usize;

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

        let effective_position = (merge_center.0 - overall_top, merge_center.1 - overall_left);

        (merged, effective_position)
    }
}

#[cfg(test)]
mod tests {
    use std::string;

    use super::*;
    use shared::utils::{print_string_matrix, string_to_strings};

    #[test]
    fn test_merge_radar_views_with_directions() {
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
        let mut map = Map::new(&radar_1);
        print_string_matrix("map + radar view 1", &map.grid);
        assert_eq!(map.position, (3, 3));

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
        print_string_matrix("radar view 2", &radar_2);
        map.merge_radar_view(&radar_2, Direction::West);
        print_string_matrix("map + radar view 2", &map.grid);
        assert_eq!(map.position, (3, 3));

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
        map.merge_radar_view(&radar_3, Direction::East);
        print_string_matrix("map + radar view 3", &map.grid);
        assert_eq!(map.position, (3, 5));

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
        map.merge_radar_view(&radar_4, Direction::East);
        print_string_matrix("map + radar view 4", &map.grid);
        assert_eq!(map.position, (3, 7));

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
        map.merge_radar_view(&radar_5, Direction::South);
        print_string_matrix("map + radar view 5", &map.grid);
        assert_eq!(map.position, (5, 7));

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
        map.merge_radar_view(&radar_6, Direction::North);
        print_string_matrix("map + radar view 6", &map.grid);
        assert_eq!(map.position, (3, 7));

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
        map.merge_radar_view(&radar_7, Direction::North);
        print_string_matrix("map + radar view 7", &map.grid);
        assert_eq!(map.position, (3, 7));

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
        map.merge_radar_view(&radar_8, Direction::South);
        print_string_matrix("map + radar view 8", &map.grid);
        assert_eq!(map.position, (5, 7));

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
}
