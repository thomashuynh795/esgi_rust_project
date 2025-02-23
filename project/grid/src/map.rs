use crate::radar::Orientation;
use shared::log_info;

pub struct Map {
    pub position: (isize, isize),
    pub matrix: Vec<Vec<String>>,
}

impl Map {
    pub fn new(encoded_first_radar_view: &str) -> Result<Map, &'static str> {
        log_info!("Player is spwaning");

        // let matrix: Vec<Vec<String>> =
        // get_readable_radar_view(encoded_first_radar_view, Orientation::North);

        let map = Map {
            position: (1, 1),
            matrix: vec![],
        };

        log_info!("Player has spawned");

        return Ok(map);
    }

    pub fn print(&self) {
        log_info!("Player is printing map");

        for row in &self.matrix {
            log_info!("{}", row.join(" "));
        }

        log_info!("Player has printed map");
    }
}
