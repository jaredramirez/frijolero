use bevy::prelude::*;

#[derive(Clone, Eq, PartialEq, Debug, Default, Component)]
pub enum Jumper {
    #[default]
    NotJumping,
    Jumping(Jumping),
}
impl Jumper {
    pub fn is_jumping(&self) -> bool {
        match self {
            Self::NotJumping => false,
            Self::Jumping(_) => true,
        }
    }

    pub fn mk_jumping() -> Self {
        Jumper::Jumping(Jumping { jumps_left: 1 })
    }
    pub fn mk_not_jumping() -> Self {
        Jumper::NotJumping
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Default)]
pub struct Jumping {
    pub jumps_left: i8,
}
