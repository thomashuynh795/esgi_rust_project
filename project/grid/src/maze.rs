use shared::types::action::RelativeDirection;

pub struct MazeState {
    pub cells: Vec<Vec<Option<bool>>>,
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
            if nx < 0 || ny < 0 || width as isize <= nx || height as isize <= ny {
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

        return None;
    }
}

pub fn choose_next_move(maze: &mut MazeState) -> Option<RelativeDirection> {
    maze.next_move_tremaux()
}
