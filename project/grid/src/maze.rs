use std::io;
use std::net::TcpStream;

use shared::types::action::{Action, RelativeDirection};
use shared::types::message::GameMessage;
use shared::types::radar::{Entity, RadarItem};
use shared::utils::decode_base64;
use shared::{log_debug, log_error, log_info, log_warning};

pub struct MazeState {
    pub cells: Vec<Vec<Option<bool>>>,
    pub horizontal_walls: Vec<Vec<Option<bool>>>,
    pub vertical_walls: Vec<Vec<Option<bool>>>,
    pub radar_cells: Vec<Vec<Option<RadarItem>>>,
    pub visits: Vec<Vec<u32>>,
    pub position: (usize, usize),
    pub orientation: RelativeDirection,
}

const SEPARATION_BAR: &str = "==========================================================";

impl MazeState {
    pub fn display(&self) {
        println!("Maze map:");
        for row in &self.cells {
            for cell in row {
                match cell {
                    Some(true) => print!("█"),
                    Some(false) => print!(" "),
                    None => print!("?"),
                }
            }
            println!();
        }
        println!("{}", SEPARATION_BAR);
    }

    pub fn new(width: usize, height: usize, orientation: RelativeDirection) -> Self {
        MazeState {
            cells: vec![vec![None; width]; height],
            horizontal_walls: vec![vec![None; width]; height + 1],
            vertical_walls: vec![vec![None; width + 1]; height],
            radar_cells: vec![vec![None; width]; height],
            visits: vec![vec![0; width]; height],
            position: (width / 2, height / 2),
            orientation,
        }
    }

    pub fn integrate_radar_data(&mut self, radar_b64: &str) {
        let decoded: Vec<u8> = decode_base64(radar_b64).expect("Base64 invalide");
        if decoded.len() != 11 {
            panic!("RadarView attend 11 octets, reçu {}", decoded.len());
        }

        let h_walls: Vec<Vec<Option<bool>>> = decode_walls(&decoded[0..3], 4, 3);
        let v_walls: Vec<Vec<Option<bool>>> = decode_walls(&decoded[3..6], 3, 4);
        let r_cells: Vec<Vec<Option<RadarItem>>> = decode_cells(&decoded[6..11]);

        let (px, py) = (self.position.0 as isize, self.position.1 as isize);
        let global_top_left_x = px - 1;
        let global_top_left_y = py - 1;

        for i in 0..3 {
            for j in 0..3 {
                let gx_signed: isize = global_top_left_x + j as isize;
                let gy_signed: isize = global_top_left_y + i as isize;
                if gx_signed >= 0
                    && gy_signed >= 0
                    && (gx_signed as usize) < self.cells[0].len()
                    && (gy_signed as usize) < self.cells.len()
                {
                    let gx: usize = gx_signed as usize;
                    let gy: usize = gy_signed as usize;

                    if self.cells[gy][gx].is_none() || r_cells[i][j].is_some() {
                        self.cells[gy][gx] = Some(match r_cells[i][j] {
                            Some(item) if item.is_goal || item.entity == Some(Entity::Monster) => {
                                true
                            }
                            _ => false,
                        });
                        self.radar_cells[gy][gx] = r_cells[i][j];
                    }
                }
            }
        }

        for i in 0..4 {
            for j in 0..3 {
                let gx_signed: isize = global_top_left_x + j as isize;
                let gy_signed: isize = global_top_left_y + i as isize;
                if gx_signed >= 0
                    && gy_signed >= 0
                    && (gy_signed as usize) < self.horizontal_walls.len()
                    && (gx_signed as usize) < self.horizontal_walls[gy_signed as usize].len()
                {
                    let gx: usize = gx_signed as usize;
                    let gy: usize = gy_signed as usize;
                    if self.horizontal_walls[gy][gx].is_none() || h_walls[i][j].is_some() {
                        self.horizontal_walls[gy][gx] = h_walls[i][j];
                    }
                }
            }
        }

        for i in 0..3 {
            for j in 0..4 {
                let gx_signed: isize = global_top_left_x + j as isize;
                let gy_signed: isize = global_top_left_y + i as isize;
                if gx_signed >= 0
                    && gy_signed >= 0
                    && (gy_signed as usize) < self.vertical_walls.len()
                    && (gx_signed as usize) < self.vertical_walls[gy_signed as usize].len()
                {
                    let gx: usize = gx_signed as usize;
                    let gy: usize = gy_signed as usize;
                    if self.vertical_walls[gy][gx].is_none() || v_walls[i][j].is_some() {
                        self.vertical_walls[gy][gx] = v_walls[i][j];
                    }
                }
            }
        }
    }

    pub fn next_move_tremaux(&mut self) -> Option<RelativeDirection> {
        let (x, y) = self.position;
        let width: usize = self.cells[0].len();
        let height: usize = self.cells.len();

        let directions: [(RelativeDirection, (isize, isize)); 4] = [
            (RelativeDirection::Front, (0, -1)),
            (RelativeDirection::Right, (1, 0)),
            (RelativeDirection::Back, (0, 1)),
            (RelativeDirection::Left, (-1, 0)),
        ];

        let mut best_dir: Option<RelativeDirection> = None;
        let mut best_visits: u32 = u32::MAX;

        for (dir, (dx, dy)) in directions.iter() {
            let nx: isize = x as isize + dx;
            let ny: isize = y as isize + dy;
            if nx < 0 || ny < 0 || nx >= width as isize || ny >= height as isize {
                continue;
            }
            let nx: usize = nx as usize;
            let ny: usize = ny as usize;

            if let Some(true) = self.cells[ny][nx] {
                continue;
            }
            let visits: u32 = self.visits[ny][nx];
            if visits < best_visits {
                best_visits = visits;
                best_dir = Some(*dir);
            }
        }

        if let Some(dir) = best_dir {
            let (dx, dy) = match dir {
                RelativeDirection::Front => (0, -1),
                RelativeDirection::Right => (1, 0),
                RelativeDirection::Back => (0, 1),
                RelativeDirection::Left => (-1, 0),
            };
            let new_x: usize = (x as isize + dx) as usize;
            let new_y: usize = (y as isize + dy) as usize;
            self.position = (new_x, new_y);
            self.visits[new_y][new_x] += 1;
            return Some(dir);
        }

        None
    }
}

fn extract_bits(data: &[u8], bit_index: usize, length: usize) -> u8 {
    let byte_index: usize = bit_index / 8;
    let bit_offset: usize = bit_index % 8;

    let mut value: u8 = (data[byte_index] >> bit_offset) & (0xFF >> (8 - length));
    if bit_offset + length > 8 {
        value |=
            (data[byte_index + 1] & (0xFF >> (16 - (bit_offset + length)))) << (8 - bit_offset);
    }
    value & ((1 << length) - 1)
}

fn decode_walls(data: &[u8], rows: usize, cols: usize) -> Vec<Vec<Option<bool>>> {
    let mut walls: Vec<Vec<Option<bool>>> = vec![vec![None; cols]; rows];
    let mut bit_index: usize = 0;
    for i in 0..rows {
        for j in 0..cols {
            let bits = extract_bits(data, bit_index, 2);
            walls[i][j] = match bits {
                0 => None,
                1 => Some(false),
                2 => Some(true),
                _ => None,
            };
            bit_index += 2;
        }
    }
    return walls;
}

fn decode_cells(data: &[u8]) -> Vec<Vec<Option<RadarItem>>> {
    let mut cells: Vec<Vec<Option<RadarItem>>> = vec![vec![None; 3]; 3];
    let mut bit_index: usize = 0;
    for i in 0..3 {
        for j in 0..3 {
            let bits = extract_bits(data, bit_index, 4);
            cells[i][j] = parse_radar_item(bits);
            bit_index += 4;
        }
    }
    return cells;
}

fn parse_radar_item(bits: u8) -> Option<RadarItem> {
    let marker: u8 = (bits & 0b1100) >> 2;
    let entity_bits: u8 = bits & 0b0011;

    if bits == 0b1111 {
        return None;
    }

    let is_hint: bool = marker == 0b01;
    let is_goal: bool = (marker == 0b10) && (entity_bits == 0b00);

    let entity: Option<Entity> = match entity_bits {
        0b01 => Some(Entity::Ally),
        0b10 => Some(Entity::Enemy),
        0b11 => Some(Entity::Monster),
        _ => None,
    };

    println!(
        "Parsed cell: {:04b} => marker={:02b}, entity_bits={:02b}, is_goal={}, entity={:?}",
        bits, marker, entity_bits, is_goal, entity
    );

    Some(RadarItem {
        is_hint,
        is_goal,
        entity,
    })
}

pub fn choose_next_move(maze: &mut MazeState) -> Option<RelativeDirection> {
    maze.next_move_tremaux()
}

pub fn send_and_receive(
    stream: &mut TcpStream,
    direction: RelativeDirection,
    maze: &mut MazeState,
) -> io::Result<()> {
    let action: GameMessage = GameMessage::Action(Action::MoveTo(direction));
    action.send(stream)?;
    log_info!("Movement sent: {:?}", direction);

    match GameMessage::receive(stream) {
        Ok(GameMessage::RadarView(data)) => {
            log_info!("RadarView received: {}", data);
            display_radar_grid_from_base64(&data);
            maze.integrate_radar_data(&data);
            maze.display();
        }
        Ok(GameMessage::ActionError(err)) => {
            log_warning!("Action error received: {:?}", err);
            if let shared::types::error::ActionError::CannotPassThroughWall = err {
                let (x, y) = maze.position;
                let (dx, dy) = match direction {
                    RelativeDirection::Front => (0, -1),
                    RelativeDirection::Right => (1, 0),
                    RelativeDirection::Back => (0, 1),
                    RelativeDirection::Left => (-1, 0),
                };
                let target_x: isize = x as isize + dx;
                let target_y: isize = y as isize + dy;

                if target_x >= 0
                    && target_y >= 0
                    && (target_x as usize) < maze.cells[0].len()
                    && (target_y as usize) < maze.cells.len()
                {
                    maze.cells[target_y as usize][target_x as usize] = Some(true);
                }
            }
        }
        Ok(other) => {
            log_warning!("Unexpected message received: {:?}", other);
        }
        Err(e) => {
            log_error!("Error receiving response: {}", e);
        }
    }
    return Ok(());
}

pub fn display_radar_grid_from_base64(radar_b64: &str) {
    let decoded: Vec<u8> = match decode_base64(radar_b64) {
        Ok(bytes) => bytes,
        Err(_) => {
            log_error!("Invalid base64 RadarView string");
            return;
        }
    };
    if decoded.len() != 11 {
        log_error!(
            "Error: RadarView expects 11 bytes, but received {}.",
            decoded.len()
        );
        return;
    }
    let r_cells: Vec<Vec<Option<RadarItem>>> = decode_cells(&decoded[6..11]);
    log_debug!("Radar view:");
    for row in r_cells.iter() {
        for cell in row.iter() {
            match cell {
                Some(RadarItem { is_goal: true, .. }) => print!("G "),
                Some(RadarItem {
                    entity: Some(Entity::Monster),
                    ..
                }) => print!("M "),
                Some(RadarItem {
                    entity: Some(Entity::Ally),
                    ..
                }) => print!("A "),
                Some(RadarItem {
                    entity: Some(Entity::Enemy),
                    ..
                }) => print!("E "),
                Some(RadarItem { is_hint: true, .. }) => print!("H "),
                Some(RadarItem {
                    is_hint: false,
                    is_goal: false,
                    entity: None,
                }) => print!(". "),
                None => print!(". "),
            }
        }
        println!();
    }
}
