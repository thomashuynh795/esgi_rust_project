use shared::log_info;

pub struct Map {
    pub position: (isize, isize),
    pub grid: Vec<Vec<String>>,
}

impl Map {
    pub fn new(encoded_first_radar_view: &str) -> Result<Map, &'static str> {
        log_info!("Player is spwaning");

        let map = Map {
            position: (1, 1),
            grid: vec![],
        };

        log_info!("Player has spawned");

        return Ok(map);
    }

    fn merge_radar_view_to_map_grid(
        saved: &Vec<Vec<String>>,
        new_view: &Vec<Vec<String>>,
        merge_center: (usize, usize),
    ) -> Vec<Vec<String>> {
        let view_size: usize = new_view.len();
        let half: usize = view_size / 2;

        let new_top: isize = merge_center.0 as isize - half as isize;
        let new_left: isize = merge_center.1 as isize - half as isize;
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
        let new_cols: usize = (overall_right - overall_left + 1) as usize;

        let mut merged: Vec<Vec<String>> = vec![vec![String::from("#"); new_cols]; new_rows];

        let offset_row: isize = -overall_top;
        let offset_col: isize = -overall_left;
        for i in 0..saved_rows as usize {
            for j in 0..saved_cols as usize {
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

        for i in 0..merged.len() {
            for j in 0..merged[i].len() {
                if merged[i][j] == "P" {
                    merged[i][j] = String::from(" ");
                }
            }
        }

        return merged;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use shared::utils::{print_string_matrix, string_to_strings};

    #[test]
    fn test_merge_radar_views() {
        let radar_1: Vec<Vec<String>> = vec![
            string_to_strings("#######"),
            string_to_strings("#######"),
            string_to_strings("##•-•-•"),
            string_to_strings("##|P   "),
            string_to_strings("##• •-•"),
            string_to_strings("##|  A "),
            string_to_strings("##•-•-•"),
        ];
        print_string_matrix("radar 1", &radar_1);

        let radar_2: Vec<Vec<String>> = vec![
            string_to_strings("##### •"),
            string_to_strings("#####M|"),
            string_to_strings("•-•-• •"),
            string_to_strings("|  P  |"),
            string_to_strings("• •-• •"),
            string_to_strings("|  A   "),
            string_to_strings("•-•-•-•"),
        ];
        print_string_matrix("radar 2", &radar_2);

        let radar_3: Vec<Vec<String>> = vec![
            string_to_strings("##• •##"),
            string_to_strings("##|M|##"),
            string_to_strings("•-• •##"),
            string_to_strings("   P|##"),
            string_to_strings("•-• •##"),
            string_to_strings(" A   ##"),
            string_to_strings("•-•-•-•"),
        ];
        print_string_matrix("radar 3", &radar_3);

        let radar_4: Vec<Vec<String>> = vec![
            string_to_strings("•-• •##"),
            string_to_strings("    |##"),
            string_to_strings("•-• •-•"),
            string_to_strings(" A P   "),
            string_to_strings("•-•-•-•"),
            string_to_strings("#######"),
            string_to_strings("#######"),
        ];
        print_string_matrix("radar 4", &radar_4);

        let expected: Vec<Vec<String>> = vec![
            string_to_strings("######• •##"),
            string_to_strings("######|M|##"),
            string_to_strings("##•-•-• •##"),
            string_to_strings("##|     |##"),
            string_to_strings("##• •-• •-•"),
            string_to_strings("##|  A     "),
            string_to_strings("##•-•-•-•-•"),
            string_to_strings("###########"),
            string_to_strings("###########"),
        ];

        let mut saved: Vec<Vec<String>> = radar_1;
        print_string_matrix("saved 1", &saved);
        saved = Map::merge_radar_view_to_map_grid(&saved, &radar_2, (3, 5));
        print_string_matrix("saved 2", &saved);
        saved = Map::merge_radar_view_to_map_grid(&saved, &radar_3, (3, 7));
        print_string_matrix("saved 3", &saved);
        saved = Map::merge_radar_view_to_map_grid(&saved, &radar_4, (5, 7));
        print_string_matrix("saved 4", &saved);

        print_string_matrix("expected radar 1 + radar 2 + radar 3 + radar 4", &expected);

        assert_eq!(saved, expected);
    }
}
