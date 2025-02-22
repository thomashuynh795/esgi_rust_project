use shared::{log_error, log_info, log_warning, utils::decode_base64};

#[derive(Debug, Clone, Copy)]
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
    let matrix: Vec<Vec<String>> = decode_radar_view(encoded_radar_view);
    return rotate_radar_view(&matrix, orientation);
}

fn decode_radar_view(encoded_radar_view: &str) -> Vec<Vec<String>> {
    let decoded: Vec<u8> =
        decode_base64(encoded_radar_view).expect("Invalid Base64 data for RadarView");

    if decoded.len() != 11 {
        log_error!(
            "RadarView expects 11 bytes, but has {} byte(s)",
            decoded.len()
        );
    }

    let cell_bytes: &[u8] = &decoded[6..11];
    let radar_cells: Vec<Vec<Option<RadarItem>>> = decode_cells(cell_bytes);

    let mut matrix: Vec<Vec<String>> = Vec::new();
    for row in 0..3 {
        let mut row_vec: Vec<String> = Vec::new();
        for col in 0..3 {
            row_vec.push(radar_item_to_string(radar_cells[row][col]));
        }
        matrix.push(row_vec);
    }

    matrix[1][1] = "P".to_string();

    return matrix;
}

fn decode_cells(data: &[u8]) -> Vec<Vec<Option<RadarItem>>> {
    let mut cells: Vec<Vec<Option<RadarItem>>> = vec![vec![None; 3]; 3];

    let mut bit_index: usize = 0;
    for row in 0..3 {
        for col in 0..3 {
            let bits: u8 = extract_bits(data, bit_index, 4);
            bit_index += 4;
            cells[row][col] = parse_radar_item(bits);
        }
    }
    return cells;
}

fn extract_bits(data: &[u8], bit_index: usize, length: usize) -> u8 {
    let byte_index: usize = bit_index / 8;
    let bit_offset: usize = bit_index % 8;

    let mut value: u8 = (data[byte_index] >> bit_offset) & ((1 << length) - 1);

    let bits_used: usize = 8 - bit_offset;
    if bits_used < length && byte_index + 1 < data.len() {
        let remaining_bits: usize = length - bits_used;
        let next_byte_mask: u8 = (1 << remaining_bits) - 1;
        let next_value: u8 = data[byte_index + 1] & next_byte_mask;
        value |= next_value << bits_used;
    }

    return value;
}

fn parse_radar_item(bits: u8) -> Option<RadarItem> {
    if bits == 0b1111 {
        return None;
    }

    let is_hint = (bits & 0b1100) == 0b0100; // Check if hint bit pattern is correct
    let is_goal = (bits & 0b1100) == 0b1000; // Check if goal bit pattern is correct

    // Ensure goal detection is STRICT (0b1000 and nothing else)
    if is_goal && (bits & 0b0011) != 0b00 {
        log_warning!(
            "⚠️ Incorrect 'G' detection prevented for bits: {:08b}",
            bits
        );
        return None;
    }

    let ent_code = bits & 0b0011;
    let entity = match ent_code {
        0b00 => None, // No entity
        0b01 => Some(Entity::Ally),
        0b10 => Some(Entity::Enemy),
        0b11 => Some(Entity::Monster),
        _ => None,
    };

    if is_goal {
        log_info!("🟢 Goal correctly detected in bits: {:08b}", bits);
    }

    Some(RadarItem {
        is_hint,
        is_goal,
        entity,
    })
}

fn radar_item_to_string(item: Option<RadarItem>) -> String {
    match item {
        None => return "?".to_string(),
        Some(it) => {
            if it.is_goal {
                return "G".to_string();
            } else if it.is_hint {
                return "H".to_string();
            } else {
                match it.entity {
                    Some(Entity::Ally) => return "A".to_string(),
                    Some(Entity::Enemy) => return "E".to_string(),
                    Some(Entity::Monster) => return "M".to_string(),
                    None => " ".to_string(),
                }
            }
        }
    }
}

pub fn rotate_radar_view(matrix: &Vec<Vec<String>>, orientation: Orientation) -> Vec<Vec<String>> {
    match orientation {
        Orientation::North => return matrix.clone(),
        Orientation::East => {
            return rotate_90_clockwise(&rotate_90_clockwise(&rotate_90_clockwise(matrix)));
        }
        Orientation::South => return rotate_90_clockwise(&rotate_90_clockwise(matrix)),
        Orientation::West => return rotate_90_clockwise(matrix),
    }
}

fn rotate_90_clockwise(matrix: &Vec<Vec<String>>) -> Vec<Vec<String>> {
    // Creates a 3x3 matrix with "?" value for each element.
    let mut rotated_matrix: Vec<Vec<String>> = vec![vec!["?".to_string(); 3]; 3];

    for row in 0..3 {
        for col in 0..3 {
            rotated_matrix[col][2 - row] = matrix[row][col].clone();
        }
    }

    return rotated_matrix;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_radar_view() {
        let encoded: &str = "ieysGjGO8papd/a";
        let matrix: Vec<Vec<String>> = decode_radar_view(encoded);

        assert_eq!(matrix.len(), 3);
        assert_eq!(matrix[0].len(), 3);

        for row in &matrix {
            println!("{:?}", row);
        }
    }

    #[test]
    fn test_get_readable_radar_view() {
        let encoded = "ieysGjGO8papd/a"; // Example Base64 data
        let matrix = get_readable_radar_view(encoded, Orientation::North);

        assert_eq!(matrix.len(), 3);
        assert_eq!(matrix[0].len(), 3);
    }

    #[test]
    fn test_decode_cells() {
        let encoded = "ieysGjGO8papd/a";
        let decoded_bytes = decode_base64(encoded).expect("Invalid Base64 data");

        let cell_bytes = &decoded_bytes[6..11];
        let radar_cells = decode_cells(cell_bytes);

        assert_eq!(radar_cells.len(), 3);
        assert_eq!(radar_cells[0].len(), 3);
    }

    #[test]
    fn test_extract_bits() {
        let data = vec![0b10101010, 0b11001100];
        let bits = extract_bits(&data, 0, 4);
        assert_eq!(bits, 0b1010, "Extracted incorrect bits");

        let bits = extract_bits(&data, 4, 4);
        assert_eq!(bits, 0b1010, "Extracted incorrect bits from next part");

        let bits = extract_bits(&data, 8, 4);
        assert_eq!(bits, 0b1100, "Extracted incorrect bits from second byte");
    }

    // #[test]
    // fn test_parse_radar_item() {
    //     let item = parse_radar_item(0b1001);
    //     assert!(item.is_some());
    //     let item = item.unwrap();
    //     assert!(item.is_goal);
    //     assert_eq!(item.entity, Some(Entity::Ally));

    //     let item = parse_radar_item(0b1111);
    //     assert!(item.is_none());
    // }

    #[test]
    fn test_radar_item_to_string() {
        let item = Some(RadarItem {
            is_hint: false,
            is_goal: true,
            entity: None,
        });
        assert_eq!(radar_item_to_string(item), "G");

        let item = Some(RadarItem {
            is_hint: true,
            is_goal: false,
            entity: None,
        });
        assert_eq!(radar_item_to_string(item), "H");

        let item = Some(RadarItem {
            is_hint: false,
            is_goal: false,
            entity: Some(Entity::Ally),
        });
        assert_eq!(radar_item_to_string(item), "A");

        let item = None;
        assert_eq!(radar_item_to_string(item), "?");
    }

    #[test]
    fn test_rotate_90_clockwise() {
        let matrix = vec![
            vec!["A".to_string(), "B".to_string(), "C".to_string()],
            vec!["D".to_string(), "E".to_string(), "F".to_string()],
            vec!["G".to_string(), "H".to_string(), "I".to_string()],
        ];

        let expected = vec![
            vec!["G".to_string(), "D".to_string(), "A".to_string()],
            vec!["H".to_string(), "E".to_string(), "B".to_string()],
            vec!["I".to_string(), "F".to_string(), "C".to_string()],
        ];

        let rotated = rotate_90_clockwise(&matrix);
        assert_eq!(rotated, expected, "90° Clockwise rotation failed!");
    }

    #[test]
    fn test_rotate_90_counterclockwise() {
        let matrix = vec![
            vec!["A".to_string(), "B".to_string(), "C".to_string()],
            vec!["D".to_string(), "E".to_string(), "F".to_string()],
            vec!["G".to_string(), "H".to_string(), "I".to_string()],
        ];

        let expected = vec![
            vec!["C".to_string(), "F".to_string(), "I".to_string()],
            vec!["B".to_string(), "E".to_string(), "H".to_string()],
            vec!["A".to_string(), "D".to_string(), "G".to_string()],
        ];

        let rotated = rotate_90_clockwise(&rotate_90_clockwise(&rotate_90_clockwise(&matrix)));
        assert_eq!(rotated, expected, "90° Counterclockwise rotation failed!");
    }

    #[test]
    fn test_rotate_180() {
        let matrix = vec![
            vec!["A".to_string(), "B".to_string(), "C".to_string()],
            vec!["D".to_string(), "E".to_string(), "F".to_string()],
            vec!["G".to_string(), "H".to_string(), "I".to_string()],
        ];

        let expected = vec![
            vec!["I".to_string(), "H".to_string(), "G".to_string()],
            vec!["F".to_string(), "E".to_string(), "D".to_string()],
            vec!["C".to_string(), "B".to_string(), "A".to_string()],
        ];

        let rotated = rotate_90_clockwise(&rotate_90_clockwise(&matrix));
        assert_eq!(rotated, expected, "180° rotation failed!");
    }
}
