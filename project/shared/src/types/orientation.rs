use super::action::RelativeDirection;

#[derive(Clone, Copy, Debug)]
pub enum Orientation {
    North,
    East,
    South,
    West,
}

impl Orientation {
    pub fn turn_right(self) -> Self {
        match self {
            Orientation::North => Orientation::East,
            Orientation::East => Orientation::South,
            Orientation::South => Orientation::West,
            Orientation::West => Orientation::North,
        }
    }

    pub fn turn_left(self) -> Self {
        match self {
            Orientation::North => Orientation::West,
            Orientation::West => Orientation::South,
            Orientation::South => Orientation::East,
            Orientation::East => Orientation::North,
        }
    }

    pub fn turn_back(self) -> Self {
        match self {
            Orientation::North => Orientation::South,
            Orientation::South => Orientation::North,
            Orientation::East => Orientation::West,
            Orientation::West => Orientation::East,
        }
    }

    pub fn to_movement(self, dir: RelativeDirection) -> (isize, isize) {
        match (self, dir) {
            (Orientation::North, RelativeDirection::Front) => (0, -1),
            (Orientation::North, RelativeDirection::Right) => (1, 0),
            (Orientation::North, RelativeDirection::Back) => (0, 1),
            (Orientation::North, RelativeDirection::Left) => (-1, 0),

            (Orientation::East, RelativeDirection::Front) => (1, 0),
            (Orientation::East, RelativeDirection::Right) => (0, 1),
            (Orientation::East, RelativeDirection::Back) => (-1, 0),
            (Orientation::East, RelativeDirection::Left) => (0, -1),

            (Orientation::South, RelativeDirection::Front) => (0, 1),
            (Orientation::South, RelativeDirection::Right) => (-1, 0),
            (Orientation::South, RelativeDirection::Back) => (0, -1),
            (Orientation::South, RelativeDirection::Left) => (1, 0),

            (Orientation::West, RelativeDirection::Front) => (-1, 0),
            (Orientation::West, RelativeDirection::Right) => (0, -1),
            (Orientation::West, RelativeDirection::Back) => (1, 0),
            (Orientation::West, RelativeDirection::Left) => (0, 1),
        }
    }
}
