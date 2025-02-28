#[derive(Debug, Clone, Copy)]
pub struct RadarItem {
    pub is_hint: bool,
    pub is_goal: bool,
    pub entity: Option<Entity>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Entity {
    Ally,
    Enemy,
    Monster,
}
