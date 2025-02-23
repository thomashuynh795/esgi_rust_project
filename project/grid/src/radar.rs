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
    // let horizontal_walls = decode_walls(h_walls_data, 4, 3);

    let v_walls_data: &[u8] = &decoded[3..6];
    // let vertical_walls = decode_walls(v_walls_data, 3, 4);

    let cell_data: &[u8] = &decoded[6..11];
    log_debug!("Extracted Cell Bytes: {:?}", cell_data);

    let radar_cells: Vec<Vec<Option<RadarItem>>> = decode_cells(cell_data);

    print_extracted_radar_bits(&decoded);
    // print_radar_raw_grid(&horizontal_walls, &vertical_walls, &radar_cells);

    // build_server_like_debug_view(&horizontal_walls, &vertical_walls, &radar_cells)
    return vec![];
}

fn build_server_like_debug_view(
    h_walls: &Vec<Vec<Option<bool>>>,
    v_walls: &Vec<Vec<Option<bool>>>,
    radar_cells: &Vec<Vec<Option<RadarItem>>>,
) -> Vec<Vec<String>> {
    let mut grid = vec![vec![" ".to_string(); 7]; 7];

    for row in 0..3 {
        for col in 0..3 {
            let rr = 2 * row + 1;
            let cc = 2 * col + 1;
            if radar_cells[row][col].is_some() {
                grid[rr][cc] = "•".to_string();
            }
        }
    }

    for hr in 0..4 {
        for hc in 0..3 {
            let rr = hr * 2;
            let cc = hc * 2 + 1;
            if let Some(true) = h_walls[hr][hc] {
                grid[rr][cc] = "-".to_string();
            }
        }
    }

    for vr in 0..3 {
        for vc in 0..4 {
            let rr = 2 * vr + 1;
            let cc = 2 * vc;
            if let Some(true) = v_walls[vr][vc] {
                grid[rr][cc] = "|".to_string();
            }
        }
    }

    grid[3][3] = "P".to_string();

    for r in 0..7 {
        for c in 0..7 {
            if grid[r][c] == " " {
                if (r == 0 || r == 6) || (c == 0 || c == 6) {
                    grid[r][c] = "#".to_string();
                }
            }
        }
    }

    return grid;
}

fn extract_bits(data: &[u8], bit_index: usize, length: usize) -> u8 {
    let byte_index: usize = bit_index / 8;
    let bit_offset: usize = bit_index % 8;

    let mut value: u8 = (data[byte_index] >> bit_offset) & ((1 << length) - 1);

    if bit_offset + length > 8 {
        value |=
            (data[byte_index + 1] & ((1 << (bit_offset + length - 8)) - 1)) << (8 - bit_offset);
    }

    return value;
}

fn print_extracted_radar_bits(decoded: &Vec<u8>) {
    log_debug!("Extracted Bits:");

    let mut bit_index = 0;

    for (_, byte) in decoded.iter().enumerate() {
        for bit_shift in (0..8).rev().step_by(2) {
            let bits = (byte >> bit_shift) & 0b11;
            log_debug!("Bit [{:02}-{:02}]: {:02b}", bit_index, bit_index + 2, bits);
            bit_index += 2;
        }
    }
}

fn walls_to_bitstring(data: &[u8]) -> String {
    if data.len() != 3 {
        log_error!("Wall data expects 3 bytes, but has {} byte(s)", data.len());
        return String::new();
    }

    log_debug!(
        "Byte of data (Little Endian)  : {:08b} {:08b} {:08b}",
        data[0],
        data[1],
        data[2]
    );

    let raw_bits: u32 = ((data[2] as u32) << 16) | ((data[1] as u32) << 8) | (data[0] as u32);

    log_debug!("Concatenated 24-bit value: {:024b}", raw_bits);

    let bit_string = format!("{:024b}", raw_bits);

    log_debug!("Bit String Representation: {}", bit_string);

    return bit_string;
}

fn extract_walls_from_string(bit_string: &str) -> Vec<Option<bool>> {
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

fn extract_cell_bits(bytes: &[u8]) -> Vec<String> {
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

fn decode_cells(data: &[u8]) -> Vec<Vec<Option<RadarItem>>> {
    let mut cells: Vec<Vec<Option<RadarItem>>> = vec![vec![None; 3]; 3];
    let mut bit_index: usize = 0;

    for row in 0..3 {
        for col in 0..3 {
            if bit_index + 4 > 36 {
                cells[row][col] = None;
                break;
            }
            let bits = extract_bits_exactly_36(data, bit_index, 4);
            bit_index += 4;

            cells[row][col] = parse_radar_item(bits);
        }
    }

    return cells;
}

fn extract_bits_exactly_36(data: &[u8], bit_index: usize, length: usize) -> u8 {
    if 36 <= bit_index {
        return 0b1111;
    }
    if 36 < bit_index + length {
        return 0b1111;
    }

    return extract_bits(data, bit_index, length);
}

fn parse_radar_item(bits: u8) -> Option<RadarItem> {
    if bits == 0b1111 {
        return None;
    }

    let nature: u8 = bits & 0b1100;
    let entity_bits: u8 = bits & 0b0011;

    let is_hint: bool = nature == 0b0100;
    let is_goal: bool = nature == 0b1000;

    let entity: Option<Entity> = match entity_bits {
        0b00 => None,
        0b01 => Some(Entity::Ally),
        0b10 => Some(Entity::Enemy),
        0b11 => Some(Entity::Monster),
        _ => None,
    };

    log_debug!(
        "Parsed Radar Item: Bits {:04b} => Hint: {}, Goal: {}, Entity: {:?}",
        bits,
        is_hint,
        is_goal,
        entity
    );

    return Some(RadarItem {
        is_hint,
        is_goal,
        entity,
    });
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

fn print_radar_raw_grid(
    h_walls: &Vec<Vec<Option<bool>>>,
    v_walls: &Vec<Vec<Option<bool>>>,
    radar_cells: &Vec<Vec<Option<RadarItem>>>,
) {
    log_debug!("Horizontal Walls (4x3) (0=open, 1=wall):");
    for row in h_walls {
        for &wall in row {
            match wall {
                Some(true) => print!("1 "),
                Some(false) => print!("0 "),
                None => print!("? "),
            }
        }
    }

    log_debug!("Vertical Walls (3x4):");
    for row in v_walls {
        for &wall in row {
            match wall {
                Some(true) => print!("1 "),
                Some(false) => print!("0 "),
                None => print!("? "),
            }
        }
    }

    log_debug!("Radar Cells (3x3) (4-bit items):");
    for row in radar_cells {
        for cell in row {
            match cell {
                Some(RadarItem { is_hint: true, .. }) => print!("H "),
                Some(RadarItem { is_goal: true, .. }) => print!("G "),
                Some(RadarItem {
                    entity: Some(Entity::Ally),
                    ..
                }) => print!("A "),
                Some(RadarItem {
                    entity: Some(Entity::Enemy),
                    ..
                }) => print!("E "),
                Some(RadarItem {
                    entity: Some(Entity::Monster),
                    ..
                }) => print!("M "),
                Some(_) => print!(". "),
                None => print!("? "),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_radar_item_no_info() {
        let item: Option<RadarItem> = parse_radar_item(0b0000);
        assert_eq!(
            item,
            Some(RadarItem {
                is_hint: false,
                is_goal: false,
                entity: None
            })
        );
    }

    #[test]
    fn test_parse_radar_item_hint_no_entity() {
        let item: Option<RadarItem> = parse_radar_item(0b0100);
        assert_eq!(
            item,
            Some(RadarItem {
                is_hint: true,
                is_goal: false,
                entity: None
            })
        );
    }

    #[test]
    fn test_extract_cell_bits() {
        let cell_data: [u8; 5] = [0xF0, 0xF0, 0x0F, 0x0F, 0xF0];
        let extracted_bits: Vec<String> = extract_cell_bits(&cell_data);

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
    /*
    11110000 11110000 00001111 00001111 11110000 little endian
    11110000 00001111 00001111 11110000 11110000 gib endian
    11110000 00001111 00001111 11110000 11110000 36 bits
    1111 0000 0000 1111 0000 1111 1111 0000 1111 9 cells
     */

    #[test]
    fn test_parse_radar_item_goal_enemy() {
        let item: Option<RadarItem> = parse_radar_item(0b1010);
        assert_eq!(
            item,
            Some(RadarItem {
                is_hint: false,
                is_goal: true,
                entity: Some(Entity::Enemy)
            })
        );
    }

    #[test]
    fn test_parse_radar_item_monster_with_hint() {
        let item: Option<RadarItem> = parse_radar_item(0b0111);
        assert_eq!(
            item,
            Some(RadarItem {
                is_hint: true,
                is_goal: false,
                entity: Some(Entity::Monster)
            })
        );
    }

    #[test]
    fn test_parse_radar_item_none() {
        let item: Option<RadarItem> = parse_radar_item(0b1111);
        assert_eq!(item, None);
    }

    #[test]
    fn test_extract_bits() {
        let data: [u8; 2] = [0b11001100, 0b10101010];
        assert_eq!(extract_bits(&data, 0, 2), 0b00);
        assert_eq!(extract_bits(&data, 2, 2), 0b11);
        assert_eq!(extract_bits(&data, 4, 4), 0b1100);
        assert_eq!(extract_bits(&data, 8, 4), 0b1010);
    }

    #[test]
    fn test_walls_to_string() {
        let horizontal_walls_bytes: [u8; 3] = [0b00100000, 0b01000110, 0b00010010];
        let vertical_walls_bytes: [u8; 3] = [0b10000000, 0b10011000, 0b00101000];

        let concatened_horizontal_walls_bits: String = walls_to_bitstring(&horizontal_walls_bytes);
        let concatened_vertical_walls_bits: String = walls_to_bitstring(&vertical_walls_bytes);

        assert_eq!(concatened_horizontal_walls_bits, "000100100100011000100000");
        assert_eq!(concatened_vertical_walls_bits, "001010001001100010000000");

        let horizontal_extracted_walls: Vec<Option<bool>> =
            extract_walls_from_string(&concatened_horizontal_walls_bits);

        let vertical_extracted_walls: Vec<Option<bool>> =
            extract_walls_from_string(&concatened_vertical_walls_bits);

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
    fn test_decode_cells() {
        let data: [u8; 5] = [0x01, 0x24, 0x8F, 0xA7, 0xD0];
        let cells: Vec<Vec<Option<RadarItem>>> = decode_cells(&data);

        for row in &cells {
            for cell in row {
                log_debug!("Decoded Cell: {:?}", cell);
            }
        }

        assert_eq!(
            cells[0][0],
            Some(RadarItem {
                is_hint: false,
                is_goal: false,
                entity: Some(Entity::Ally)
            })
        );
        assert_eq!(
            cells[0][1],
            Some(RadarItem {
                is_hint: false,
                is_goal: false,
                entity: None
            })
        );
    }

    #[test]
    fn test_build_server_like_debug_view() {
        let h_walls: Vec<Vec<Option<bool>>> = vec![
            vec![Some(true), Some(false), None],
            vec![None, Some(true), Some(false)],
            vec![Some(false), None, Some(true)],
            vec![Some(true), Some(false), None],
        ];

        let v_walls: Vec<Vec<Option<bool>>> = vec![
            vec![Some(false), None, Some(true), Some(false)],
            vec![Some(true), Some(false), None, Some(true)],
            vec![None, Some(true), Some(false), None],
        ];

        let radar_cells: Vec<Vec<Option<RadarItem>>> = vec![
            vec![
                Some(RadarItem {
                    is_hint: false,
                    is_goal: false,
                    entity: None,
                }),
                Some(RadarItem {
                    is_hint: false,
                    is_goal: false,
                    entity: Some(Entity::Ally),
                }),
                None,
            ],
            vec![
                None,
                Some(RadarItem {
                    is_hint: false,
                    is_goal: true,
                    entity: None,
                }),
                Some(RadarItem {
                    is_hint: false,
                    is_goal: false,
                    entity: Some(Entity::Enemy),
                }),
            ],
            vec![
                Some(RadarItem {
                    is_hint: false,
                    is_goal: false,
                    entity: None,
                }),
                None,
                Some(RadarItem {
                    is_hint: true,
                    is_goal: false,
                    entity: Some(Entity::Monster),
                }),
            ],
        ];

        let result: Vec<Vec<String>> =
            build_server_like_debug_view(&h_walls, &v_walls, &radar_cells);

        assert_eq!(result.len(), 7);
        assert_eq!(result[0].len(), 7);
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
