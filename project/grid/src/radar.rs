use shared::{log_debug, log_error, types::log, utils::decode_base64};

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

pub fn get_readable_radar_view(
    encoded_radar_view: &str,
    orientation: Orientation,
) -> Vec<Vec<String>> {
    let decoded_map: Vec<Vec<String>> = decode_radar_view(encoded_radar_view);
    rotate_radar_view(&decoded_map, orientation)
}

fn decode_radar_view(encoded_radar_view: &str) -> Vec<Vec<String>> {
    let decoded: Vec<u8> =
        decode_base64(encoded_radar_view).expect("Invalid Base64 data for RadarView");

    log_debug!("Full Decoded Data: {:?}", decoded);

    if decoded.len() != 11 {
        log_error!(
            "RadarView expects 11 bytes, but has {} byte(s)",
            decoded.len()
        );
    }

    for (i, byte) in decoded.iter().enumerate() {
        log_debug!("Byte {}: {:08b} ({})", i, byte, byte);
    }

    let h_walls_data: &[u8] = &decoded[0..3];
    let h_walls_bits: String = convert_walls_bytes_to_string(h_walls_data);
    let horizontal_walls: Vec<Option<bool>> = extract_walls_data_from_bits_string(&h_walls_bits);

    let v_walls_data: &[u8] = &decoded[3..6];
    let v_walls_bits: String = convert_walls_bytes_to_string(v_walls_data);
    let vertical_walls: Vec<Option<bool>> = extract_walls_data_from_bits_string(&v_walls_bits);

    let cell_data: &[u8] = &decoded[6..11];
    let cell_bits: Vec<String> = extract_cells_data(cell_data);
    let radar_items: Vec<Option<RadarItem>> = cell_bits
        .iter()
        .map(|bits: &String| get_radar_item_from_bits(bits))
        .collect();

    log_debug!("Extracted cell Bits: {:?}", cell_bits);

    let merged_view: Vec<Vec<String>> =
        build_radar_matrix(&horizontal_walls, &vertical_walls, &radar_items);

    for row in &merged_view {
        for col in row {
            log_debug!("{}", col);
        }
    }

    log_debug!("Merged radar view:");
    for row in &merged_view {
        log_debug!("{}", row.join(""));
    }

    merged_view
}

fn build_radar_matrix(
    h_walls: &Vec<Option<bool>>,
    v_walls: &Vec<Option<bool>>,
    radar_items: &Vec<Option<RadarItem>>,
) -> Vec<Vec<String>> {
    let mut matrix: Vec<Vec<String>> = vec![vec![" ".to_string(); 7]; 7];

    for i in 0..3 {
        for j in 0..3 {
            let rr: usize = 2 * i + 1;
            let cc: usize = 2 * j + 1;
            if let Some(item) = &radar_items[i * 3 + j] {
                matrix[rr][cc] = match item {
                    RadarItem { is_goal: true, .. } => "G".to_string(),
                    RadarItem { is_hint: true, .. } => "H".to_string(),
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
                    _ => "•".to_string(),
                };
            } else {
                matrix[rr][cc] = "•".to_string();
            }
        }
    }

    for i in 0..4 {
        for j in 0..3 {
            let rr: usize = 2 * i;
            let cc: usize = 2 * j + 1;
            if let Some(true) = h_walls[i * 3 + j] {
                matrix[rr][cc] = "-".to_string();
            }
        }
    }

    for i in 0..3 {
        for j in 0..4 {
            let rr: usize = 2 * i + 1;
            let cc: usize = 2 * j;
            if let Some(true) = v_walls[i * 4 + j] {
                matrix[rr][cc] = "|".to_string();
            }
        }
    }

    for r in 0..7 {
        for c in 0..7 {
            if matrix[r][c] == " " {
                if r == 0 || r == 6 || c == 0 || c == 6 {
                    matrix[r][c] = "#".to_string();
                }
            }
        }
    }

    return matrix;
}

/*=============================================================*\
    WALLS EXTRACTION
*\=============================================================*/

fn convert_walls_bytes_to_string(data: &[u8]) -> String {
    if data.len() != 3 {
        log_error!("Wall data expects 3 bytes, but has {} byte(s)", data.len());
        return String::new();
    }

    let raw_bits: u32 = ((data[2] as u32) << 16) | ((data[1] as u32) << 8) | (data[0] as u32);
    let bit_string = format!("{:024b}", raw_bits);

    return bit_string;
}

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
        "Byte of cell data (input) : {:02X} {:02X} {:02X} {:02X} {:02X}",
        bytes[0],
        bytes[1],
        bytes[2],
        bytes[3],
        bytes[4]
    );

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

pub fn rotate_radar_view(matrix: &Vec<Vec<String>>, orientation: Orientation) -> Vec<Vec<String>> {
    match orientation {
        Orientation::North => matrix.clone(),
        Orientation::East => {
            rotate_90_clockwise(&rotate_90_clockwise(&rotate_90_clockwise(matrix)))
        }
        Orientation::South => rotate_90_clockwise(&rotate_90_clockwise(matrix)),
        Orientation::West => rotate_90_clockwise(matrix),
    }
}

fn rotate_90_clockwise(matrix: &Vec<Vec<String>>) -> Vec<Vec<String>> {
    let n: usize = matrix.len();
    let mut rotated: Vec<Vec<String>> = vec![vec!["#".to_string(); n]; n];
    for r in 0..n {
        for c in 0..n {
            rotated[c][n - 1 - r] = matrix[r][c].clone();
        }
    }

    return rotated;
}

/*=============================================================*\
    TESTS
*\=============================================================*/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_matrix() {
        let radar_view: &str = "ieysGjGO8papd/a";
        get_readable_radar_view(radar_view, Orientation::North);
    }

    #[test]
    fn test_extract_cell_bits() {
        let cell_data: [u8; 5] = [0xF0, 0xF0, 0x0F, 0x0F, 0xF0];
        let extracted_bits: Vec<String> = extract_cells_data(&cell_data);

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
            convert_walls_bytes_to_string(&horizontal_walls_bytes);
        let concatened_vertical_walls_bits: String =
            convert_walls_bytes_to_string(&vertical_walls_bytes);

        assert_eq!(concatened_horizontal_walls_bits, "000100100100011000100000");
        assert_eq!(concatened_vertical_walls_bits, "001010001001100010000000");

        let horizontal_extracted_walls: Vec<Option<bool>> =
            extract_walls_data_from_bits_string(&concatened_horizontal_walls_bits);

        let vertical_extracted_walls: Vec<Option<bool>> =
            extract_walls_data_from_bits_string(&concatened_vertical_walls_bits);

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
        let input: Vec<Vec<String>> = vec![
            vec!["1".to_string(), "2".to_string(), "3".to_string()],
            vec!["4".to_string(), "5".to_string(), "6".to_string()],
            vec!["7".to_string(), "8".to_string(), "9".to_string()],
        ];

        let expected: Vec<Vec<String>> = vec![
            vec!["7".to_string(), "4".to_string(), "1".to_string()],
            vec!["8".to_string(), "5".to_string(), "2".to_string()],
            vec!["9".to_string(), "6".to_string(), "3".to_string()],
        ];

        assert_eq!(rotate_90_clockwise(&input), expected);
    }

    #[test]
    fn test_rotate_radar_view() {
        let input: Vec<Vec<String>> = vec![
            vec!["1".to_string(), "2".to_string(), "3".to_string()],
            vec!["4".to_string(), "5".to_string(), "6".to_string()],
            vec!["7".to_string(), "8".to_string(), "9".to_string()],
        ];

        let rotated_90: Vec<Vec<String>> = vec![
            vec!["7".to_string(), "4".to_string(), "1".to_string()],
            vec!["8".to_string(), "5".to_string(), "2".to_string()],
            vec!["9".to_string(), "6".to_string(), "3".to_string()],
        ];

        let result: Vec<Vec<String>> = rotate_90_clockwise(&input);

        assert_eq!(result, rotated_90);
    }
}
