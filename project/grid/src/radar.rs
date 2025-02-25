use shared::{log_debug, log_error, utils::decode_base64};

/*
Radar View:
•-•-•-•
| | | |
•-•-•-•
| |P| |
•-•-•-•
| | | |
•-•-•-•
*/

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RadarItem {
    pub is_hint: bool,
    pub is_goal: bool,
    pub entity: Option<Entity>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Entity {
    Ally,
    Enemy,
    Monster,
}

#[derive(Debug, Clone, Copy)]
pub enum Orientation {
    North,
    East,
    South,
    West,
}

pub struct RadarView {
    pub encoded_view: String,
    pub decoded_view: Vec<u8>,
    pub horizontal_walls: Vec<Vec<Option<bool>>>,
    pub vertical_walls: Vec<Vec<Option<bool>>>,
    pub radar_items: Vec<Vec<Option<RadarItem>>>,
    pub grid: Vec<Vec<String>>,
    pub orientation: Orientation,
}

impl RadarView {
    pub fn new(encoded_view: String, orientation: Orientation) -> RadarView {
        let mut radar_view: RadarView = RadarView {
            encoded_view,
            decoded_view: vec![],
            horizontal_walls: vec![],
            vertical_walls: vec![],
            radar_items: vec![],
            grid: vec![],
            orientation,
        };

        radar_view.print_encoded_view();
        radar_view.decode_view();
        radar_view.extract_data();
        radar_view.merge_walls();
        radar_view.rotate_radar_view();

        return radar_view;
    }

    pub fn merge_walls(&mut self) {
        self.grid = vec![vec!["#".to_string(); 7]; 7];

        for i in 0..4 {
            for j in 0..3 {
                match self.horizontal_walls[i][j] {
                    Some(true) => self.grid[2 * i][2 * j + 1] = "-".to_string(),
                    Some(false) => self.grid[2 * i][2 * j + 1] = " ".to_string(),
                    None => (),
                }
            }
        }

        for i in 0..3 {
            for j in 0..4 {
                match self.vertical_walls[i][j] {
                    Some(true) => self.grid[2 * i + 1][2 * j] = "|".to_string(),
                    Some(false) => self.grid[2 * i + 1][2 * j] = " ".to_string(),
                    None => (),
                }
            }
        }
        for i in 0..3 {
            for j in 0..3 {
                let wall_x = 2 * i + 1;
                let wall_y = 2 * j + 1;

                if let Some(radar_item) = &self.radar_items[i][j] {
                    self.grid[wall_x][wall_y] = match radar_item {
                        RadarItem { is_hint: true, .. } => "H".to_string(),
                        RadarItem { is_goal: true, .. } => "G".to_string(),
                        RadarItem {
                            entity: Some(Entity::Ally),
                            ..
                        } => "A".to_string(),
                        RadarItem {
                            entity: Some(Entity::Enemy),
                            ..
                        } => "E".to_string(),
                        RadarItem {
                            entity: Some(Entity::Monster),
                            ..
                        } => "M".to_string(),
                        _ => " ".to_string(),
                    };
                }
            }
        }

        for i in 0..7 {
            for j in 0..7 {
                if self.grid[i][j] == "|" {
                    self.grid[i - 1][j] = "•".to_string();
                    self.grid[i + 1][j] = "•".to_string();
                }
                if self.grid[i][j] == "-" {
                    self.grid[i][j - 1] = "•".to_string();
                    self.grid[i][j + 1] = "•".to_string();
                }
            }
        }
    }

    pub fn decode_view(&mut self) -> () {
        let decoded: Vec<u8> =
            decode_base64(&self.encoded_view).expect("Invalid Base64 data for RadarView");
        if decoded.len() != 11 {
            log_error!(
                "RadarView expects 11 bytes, but has {} byte(s)",
                decoded.len()
            );
        }
        self.decoded_view = decoded;
        self.print_decoded_view();
    }

    fn extract_data(&mut self) {
        let h_walls_data: &[u8] = &self.decoded_view[0..3];
        log_debug!("Horizontal walls data bytes:");
        for byte in h_walls_data {
            log_debug!("{:08b}", byte);
        }
        log_debug!("==============================");
        let h_walls_bits: String = RadarView::convert_walls_bytes_to_string(h_walls_data);
        self.horizontal_walls = RadarView::convert_horizontal_walls_to_matrix(
            RadarView::extract_walls_data_from_bits_string(&h_walls_bits),
        );

        let v_walls_data: &[u8] = &self.decoded_view[3..6];
        log_debug!("Vertical walls data bytes:");
        for byte in v_walls_data {
            log_debug!("{:08b}", byte);
        }
        log_debug!("==============================");
        let v_walls_bits: String = RadarView::convert_walls_bytes_to_string(v_walls_data);
        self.vertical_walls = RadarView::convert_vertical_walls_to_matrix(
            RadarView::extract_walls_data_from_bits_string(&v_walls_bits),
        );

        let cell_data: &[u8] = &self.decoded_view[6..11];
        log_debug!("Cell data bytes:");
        for byte in cell_data {
            log_debug!("{:08b}", byte);
        }
        log_debug!("==============================");
        let cell_bits: Vec<String> = RadarView::extract_cells_data(cell_data);
        self.radar_items = RadarView::convert_cells_items_to_matrix(
            cell_bits
                .iter()
                .map(|bits: &String| RadarView::get_radar_item_from_bits(bits)) // No Some()
                .collect(),
        );

        self.print_horizontal_walls();
        self.print_vertical_walls();
        self.print_cells_items();
    }

    fn convert_walls_bytes_to_string(data: &[u8]) -> String {
        if data.len() != 3 {
            log_error!("Wall data expects 3 bytes, but has {} byte(s)", data.len());
            return String::new();
        }

        let raw_bits: u32 = ((data[2] as u32) << 16) | ((data[1] as u32) << 8) | (data[0] as u32);
        let bit_string: String = format!("{:024b}", raw_bits);

        return bit_string;
    }

    fn rotate_radar_view(&mut self) -> () {
        self.grid = match self.orientation {
            Orientation::North => self.grid.clone(),
            Orientation::East => RadarView::rotate_90_clockwise(&RadarView::rotate_90_clockwise(
                &RadarView::rotate_90_clockwise(&self.grid.clone()),
            )),
            Orientation::South => {
                RadarView::rotate_90_clockwise(&RadarView::rotate_90_clockwise(&self.grid.clone()))
            }
            Orientation::West => RadarView::rotate_90_clockwise(&self.grid.clone()),
        };
    }

    fn convert_horizontal_walls_to_matrix(
        boolean_options: Vec<Option<bool>>,
    ) -> Vec<Vec<Option<bool>>> {
        let mut matrix: Vec<Vec<Option<bool>>> = vec![vec![None; 3]; 4];

        for (i, chunk) in boolean_options.chunks_exact(3).enumerate() {
            matrix[i].copy_from_slice(chunk);
        }

        return matrix;
    }

    fn convert_vertical_walls_to_matrix(
        boolean_options: Vec<Option<bool>>,
    ) -> Vec<Vec<Option<bool>>> {
        let mut matrix: Vec<Vec<Option<bool>>> = vec![vec![None; 4]; 3];

        for (i, chunk) in boolean_options.chunks_exact(4).enumerate() {
            matrix[i].copy_from_slice(chunk);
        }

        return matrix;
    }

    fn convert_cells_items_to_matrix(
        cells_items: Vec<Option<RadarItem>>,
    ) -> Vec<Vec<Option<RadarItem>>> {
        cells_items.chunks(3).map(|chunk| chunk.to_vec()).collect()
    }

    /*=============================================================*\
        PRINTERS
    *\=============================================================*/

    pub fn print_grid(&self) {
        log_debug!("Grid:");
        for row in &self.grid {
            log_debug!("{:?}", row);
        }
    }

    pub fn print_matrix<T: std::fmt::Debug>(matrix: &Vec<Vec<T>>) {
        for row in matrix {
            log_debug!("{:?}", row);
        }

        log_debug!("==============================");
    }

    pub fn print_horizontal_walls(&self) {
        log_debug!("Horizontal walls:");
        let _ = RadarView::print_matrix(&self.horizontal_walls);
    }

    pub fn print_vertical_walls(&self) {
        log_debug!("Vertical walls:");
        let _ = RadarView::print_matrix(&self.vertical_walls);
    }

    pub fn print_cells_items(&self) {
        log_debug!("Cells items:");
        let _ = RadarView::print_matrix(&self.radar_items);
    }

    pub fn print_walls(&self) {
        log_debug!("Walls:");
        let _ = RadarView::print_matrix(&self.grid);
    }

    pub fn print_encoded_view(&self) {
        log_debug!("Encoded view: {}", self.encoded_view);
        log_debug!("==============================");
    }

    pub fn print_decoded_view(&self) {
        log_debug!("Decoded view:");
        for byte in &self.decoded_view {
            log_debug!("{:08b}", byte);
        }

        log_debug!("==============================");
    }

    /*=============================================================*\
        WALLS EXTRACTION
    *\=============================================================*/

    fn extract_walls_data_from_bits_string(bit_string: &str) -> Vec<Option<bool>> {
        return bit_string
            .chars()
            .collect::<Vec<char>>()
            .chunks(2)
            .map(|chunk: &[char]| {
                let pair: String = chunk.iter().collect::<String>();
                match pair.as_str() {
                    "00" => None,
                    "01" => Some(false),
                    "10" => Some(true),
                    _ => None,
                }
            })
            .collect();
    }

    /*=============================================================*\
        CELLS EXTRACTION
    *\=============================================================*/

    fn extract_cells_data(bytes: &[u8]) -> Vec<String> {
        if bytes.len() != 5 {
            log_error!("Cell data expects 5 bytes, but has {}", bytes.len());
            return vec![];
        }

        log_debug!(
            "Byte of cell data (input) : {:0b} {:0b} {:0b} {:0b} {:0b}",
            bytes[0],
            bytes[1],
            bytes[2],
            bytes[3],
            bytes[4]
        );

        log_debug!("==============================");

        let raw_40_bits: u64 = ((bytes[0] as u64) << 32)
            | ((bytes[1] as u64) << 24)
            | ((bytes[2] as u64) << 16)
            | ((bytes[3] as u64) << 8)
            | (bytes[4] as u64);

        let full_str: String = format!("{:040b}", raw_40_bits);
        let bits_36: &str = &full_str[..36];

        let chunked: Vec<String> = bits_36
            .chars()
            .collect::<Vec<_>>()
            .chunks(4)
            .map(|c| c.iter().collect::<String>())
            .collect();

        return chunked;
    }

    fn get_radar_item_from_bits(bits: &str) -> Option<RadarItem> {
        return match bits {
            "0000" => Some(RadarItem {
                is_hint: false,
                is_goal: false,
                entity: None,
            }),
            "0100" => Some(RadarItem {
                is_hint: true,
                is_goal: false,
                entity: None,
            }),
            "1000" => Some(RadarItem {
                is_hint: false,
                is_goal: true,
                entity: None,
            }),
            "1100" => Some(RadarItem {
                is_hint: true,
                is_goal: true,
                entity: None,
            }),
            "0001" => Some(RadarItem {
                is_hint: false,
                is_goal: false,
                entity: Some(Entity::Ally),
            }),
            "0010" => Some(RadarItem {
                is_hint: false,
                is_goal: false,
                entity: Some(Entity::Enemy),
            }),
            "0011" => Some(RadarItem {
                is_hint: false,
                is_goal: false,
                entity: Some(Entity::Monster),
            }),
            _ => None,
        };
    }

    fn rotate_90_clockwise(matrix: &Vec<Vec<String>>) -> Vec<Vec<String>> {
        let n: usize = matrix.len();
        let mut rotated: Vec<Vec<String>> = vec![vec!["#".to_string(); n]; n];
        for row in 0..n {
            for column in 0..n {
                rotated[column][n - 1 - row] = matrix[row][column].clone();
            }
        }

        return rotated;
    }
}

/*=============================================================*\
    TESTS
*\=============================================================*/

#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;

    #[test]
    fn test_new() {
        let radar_view_1: RadarView =
            RadarView::new("ieysGjGO8papd/a".to_string(), Orientation::North);
        radar_view_1.print_walls();
        let expected_1: Vec<Vec<&str>> = vec![
            vec!["#", "#", "•", " ", "•", "#", "#"],
            vec!["#", "#", "|", " ", "|", "#", "#"],
            vec!["•", "-", "•", " ", "•", "#", "#"],
            vec!["|", " ", " ", " ", "|", "#", "#"],
            vec!["•", " ", "•", "-", "•", "#", "#"],
            vec!["|", " ", "#", "#", "#", "#", "#"],
            vec!["•", "-", "•", "#", "#", "#", "#"],
        ];
        log_debug!("Expected walls 1:");
        RadarView::print_matrix(&expected_1);
        assert_eq!(radar_view_1.grid, expected_1);

        let radar_view_2: RadarView =
            RadarView::new("aeiOacGM8a8p//a".to_string(), Orientation::North);
        radar_view_2.print_walls();
        let expected_2: Vec<Vec<&str>> = vec![
            vec!["#", "#", "•", "-", "•", "-", "•"],
            vec!["#", "#", "|", " ", " ", " ", "|"],
            vec!["#", "#", "•", " ", "•", "#", "#"],
            vec!["#", "#", "|", " ", "|", "#", "#"],
            vec!["#", "#", "•", "-", "•", "#", "#"],
            vec!["#", "#", "#", "#", "#", "#", "#"],
            vec!["#", "#", "#", "#", "#", "#", "#"],
        ];
        log_debug!("Expected walls 2:");
        RadarView::print_matrix(&expected_2);
        assert_eq!(radar_view_2.grid, expected_2);

        let radar_view_3: RadarView =
            RadarView::new("HweGjsyO8p8a8aa".to_string(), Orientation::North);
        radar_view_3.print_walls();
        /*
        ##•-•##
        ##| |##
        ##• •-•
        ##|   |
        ##• •-•
        ##|
        ##• • •
        */
        let expected_3: Vec<Vec<&str>> = vec![
            vec!["#", "#", "•", "-", "•", "#", "#"],
            vec!["#", "#", "|", " ", "|", "#", "#"],
            vec!["#", "#", "•", " ", "•", "-", "•"],
            vec!["#", "#", "|", " ", " ", " ", "|"],
            vec!["#", "#", "•", " ", "•", "-", "•"],
            vec!["#", "#", "|", " ", " ", " ", " "],
            vec!["#", "#", "•", " ", "•", " ", "•"],
        ];
        log_debug!("Expected walls 3:");
        RadarView::print_matrix(&expected_3);
        assert_eq!(radar_view_3.grid, expected_3);
    }

    #[test]
    fn test_build_matrix() {
        let radar_view: RadarView =
            RadarView::new("geguwcHwaa8papa".to_string(), Orientation::North);

        radar_view.print_grid();
        log_debug!("Expected radar view:");
        log_debug!("##• •##");
        log_debug!("##| |##");
        log_debug!("•-• •##");
        log_debug!("|   |##");
        log_debug!("• •-•##");
        log_debug!("| #####");
        log_debug!("•-•####");
    }

    #[test]
    fn test_extract_cell_bits() {
        let cell_data: [u8; 5] = [0xF0, 0xF0, 0x0F, 0x0F, 0xF0];
        let extracted_bits: Vec<String> = RadarView::extract_cells_data(&cell_data);

        assert_eq!(
            extracted_bits,
            vec![
                "1111".to_string(),
                "0000".to_string(),
                "1111".to_string(),
                "0000".to_string(),
                "0000".to_string(),
                "1111".to_string(),
                "0000".to_string(),
                "1111".to_string(),
                "1111".to_string()
            ]
        );
    }

    #[test]
    fn test_walls_to_string() {
        let horizontal_walls_bytes: [u8; 3] = [0b00100000, 0b01000110, 0b00010010];
        let vertical_walls_bytes: [u8; 3] = [0b10000000, 0b10011000, 0b00101000];

        let concatened_horizontal_walls_bits: String =
            RadarView::convert_walls_bytes_to_string(&horizontal_walls_bytes);
        let concatened_vertical_walls_bits: String =
            RadarView::convert_walls_bytes_to_string(&vertical_walls_bytes);

        assert_eq!(concatened_horizontal_walls_bits, "000100100100011000100000");
        assert_eq!(concatened_vertical_walls_bits, "001010001001100010000000");

        let horizontal_extracted_walls: Vec<Option<bool>> =
            RadarView::extract_walls_data_from_bits_string(&concatened_horizontal_walls_bits);

        let vertical_extracted_walls: Vec<Option<bool>> =
            RadarView::extract_walls_data_from_bits_string(&concatened_vertical_walls_bits);

        let expected_horizontal_walls: Vec<Option<bool>> = vec![
            // Line 1.
            None,
            Some(false),
            None,
            // Line 2.
            Some(true),
            Some(false),
            None,
            // Line 3.
            Some(false),
            Some(true),
            None,
            // Line 4.
            Some(true),
            None,
            None,
        ];

        let expected_vertical_walls: Vec<Option<bool>> = vec![
            // Line 1.
            None,
            Some(true),
            Some(true),
            None,
            // Line 2.
            Some(true),
            Some(false),
            Some(true),
            None,
            // Line 3.
            Some(true),
            None,
            None,
            None,
        ];

        assert_eq!(horizontal_extracted_walls, expected_horizontal_walls);
        assert_eq!(vertical_extracted_walls, expected_vertical_walls);
    }

    #[test]
    fn test_rotate_90_clockwise() {
        let radar_view: RadarView = RadarView {
            encoded_view: "".to_string(),
            decoded_view: vec![],
            horizontal_walls: vec![],
            vertical_walls: vec![],
            radar_items: vec![],
            grid: vec![
                vec![
                    "1".to_string(),
                    "2".to_string(),
                    "3".to_string(),
                    "4".to_string(),
                    "5".to_string(),
                    "6".to_string(),
                    "7".to_string(),
                ],
                vec![
                    "8".to_string(),
                    "9".to_string(),
                    "10".to_string(),
                    "11".to_string(),
                    "12".to_string(),
                    "13".to_string(),
                    "14".to_string(),
                ],
                vec![
                    "15".to_string(),
                    "16".to_string(),
                    "17".to_string(),
                    "18".to_string(),
                    "19".to_string(),
                    "20".to_string(),
                    "21".to_string(),
                ],
                vec![
                    "22".to_string(),
                    "23".to_string(),
                    "24".to_string(),
                    "25".to_string(),
                    "26".to_string(),
                    "27".to_string(),
                    "28".to_string(),
                ],
                vec![
                    "29".to_string(),
                    "30".to_string(),
                    "31".to_string(),
                    "32".to_string(),
                    "33".to_string(),
                    "34".to_string(),
                    "35".to_string(),
                ],
                vec![
                    "36".to_string(),
                    "37".to_string(),
                    "38".to_string(),
                    "39".to_string(),
                    "40".to_string(),
                    "41".to_string(),
                    "42".to_string(),
                ],
                vec![
                    "43".to_string(),
                    "44".to_string(),
                    "45".to_string(),
                    "46".to_string(),
                    "47".to_string(),
                    "48".to_string(),
                    "49".to_string(),
                ],
            ],
            orientation: Orientation::North,
        };

        let expected: Vec<Vec<String>> = vec![
            vec![
                "43".to_string(),
                "36".to_string(),
                "29".to_string(),
                "22".to_string(),
                "15".to_string(),
                "8".to_string(),
                "1".to_string(),
            ],
            vec![
                "44".to_string(),
                "37".to_string(),
                "30".to_string(),
                "23".to_string(),
                "16".to_string(),
                "9".to_string(),
                "2".to_string(),
            ],
            vec![
                "45".to_string(),
                "38".to_string(),
                "31".to_string(),
                "24".to_string(),
                "17".to_string(),
                "10".to_string(),
                "3".to_string(),
            ],
            vec![
                "46".to_string(),
                "39".to_string(),
                "32".to_string(),
                "25".to_string(),
                "18".to_string(),
                "11".to_string(),
                "4".to_string(),
            ],
            vec![
                "47".to_string(),
                "40".to_string(),
                "33".to_string(),
                "26".to_string(),
                "19".to_string(),
                "12".to_string(),
                "5".to_string(),
            ],
            vec![
                "48".to_string(),
                "41".to_string(),
                "34".to_string(),
                "27".to_string(),
                "20".to_string(),
                "13".to_string(),
                "6".to_string(),
            ],
            vec![
                "49".to_string(),
                "42".to_string(),
                "35".to_string(),
                "28".to_string(),
                "21".to_string(),
                "14".to_string(),
                "7".to_string(),
            ],
        ];

        assert_eq!(
            RadarView::rotate_90_clockwise(&radar_view.grid.clone()),
            expected
        );
    }

    #[test]
    fn test_rotate_radar_view() {
        let mut radar_view: RadarView = RadarView {
            encoded_view: "ieysGjGO8papd/a".to_string(),
            decoded_view: vec![],
            horizontal_walls: vec![],
            vertical_walls: vec![],
            radar_items: vec![],
            grid: vec![
                vec![
                    "43".to_string(),
                    "36".to_string(),
                    "29".to_string(),
                    "22".to_string(),
                    "15".to_string(),
                    "8".to_string(),
                    "1".to_string(),
                ],
                vec![
                    "44".to_string(),
                    "37".to_string(),
                    "30".to_string(),
                    "23".to_string(),
                    "16".to_string(),
                    "9".to_string(),
                    "2".to_string(),
                ],
                vec![
                    "45".to_string(),
                    "38".to_string(),
                    "31".to_string(),
                    "24".to_string(),
                    "17".to_string(),
                    "10".to_string(),
                    "3".to_string(),
                ],
                vec![
                    "46".to_string(),
                    "39".to_string(),
                    "32".to_string(),
                    "25".to_string(),
                    "18".to_string(),
                    "11".to_string(),
                    "4".to_string(),
                ],
                vec![
                    "47".to_string(),
                    "40".to_string(),
                    "33".to_string(),
                    "26".to_string(),
                    "19".to_string(),
                    "12".to_string(),
                    "5".to_string(),
                ],
                vec![
                    "48".to_string(),
                    "41".to_string(),
                    "34".to_string(),
                    "27".to_string(),
                    "20".to_string(),
                    "13".to_string(),
                    "6".to_string(),
                ],
                vec![
                    "49".to_string(),
                    "42".to_string(),
                    "35".to_string(),
                    "28".to_string(),
                    "21".to_string(),
                    "14".to_string(),
                    "7".to_string(),
                ],
            ],
            orientation: Orientation::East,
        };

        radar_view.rotate_radar_view();

        let expected_grid: Vec<Vec<String>> = vec![
            vec![
                "1".to_string(),
                "2".to_string(),
                "3".to_string(),
                "4".to_string(),
                "5".to_string(),
                "6".to_string(),
                "7".to_string(),
            ],
            vec![
                "8".to_string(),
                "9".to_string(),
                "10".to_string(),
                "11".to_string(),
                "12".to_string(),
                "13".to_string(),
                "14".to_string(),
            ],
            vec![
                "15".to_string(),
                "16".to_string(),
                "17".to_string(),
                "18".to_string(),
                "19".to_string(),
                "20".to_string(),
                "21".to_string(),
            ],
            vec![
                "22".to_string(),
                "23".to_string(),
                "24".to_string(),
                "25".to_string(),
                "26".to_string(),
                "27".to_string(),
                "28".to_string(),
            ],
            vec![
                "29".to_string(),
                "30".to_string(),
                "31".to_string(),
                "32".to_string(),
                "33".to_string(),
                "34".to_string(),
                "35".to_string(),
            ],
            vec![
                "36".to_string(),
                "37".to_string(),
                "38".to_string(),
                "39".to_string(),
                "40".to_string(),
                "41".to_string(),
                "42".to_string(),
            ],
            vec![
                "43".to_string(),
                "44".to_string(),
                "45".to_string(),
                "46".to_string(),
                "47".to_string(),
                "48".to_string(),
                "49".to_string(),
            ],
        ];

        assert_eq!(expected_grid, radar_view.grid);
    }
}
