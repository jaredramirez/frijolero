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

#[derive(Clone, Eq, PartialEq, Debug, Default, Component)]
pub enum Dasher {
    #[default]
    NotDashing,
    Dashing(Dashing),
}
impl Dasher {
    pub fn is_dashing(&self) -> bool {
        match self {
            Self::NotDashing => false,
            Self::Dashing(_) => true,
        }
    }

    pub fn mk_dashing() -> Self {
        Dasher::Dashing(Dashing { dashs_left: 1 })
    }
    pub fn mk_not_dashing() -> Self {
        Dasher::NotDashing
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Default)]
pub struct Dashing {
    pub dashs_left: i8,
}

#[derive(Debug, Clone, Copy)]
pub enum DashDirection {
    Up,
    Down,
    Right,
    Left,
    UpRight,
    DownRight,
    DownLeft,
    UpLeft,
}

impl DashDirection {
    pub fn make(
        pressed_up: bool,
        pressed_right: bool,
        pressed_down: bool,
        pressed_left: bool,
        is_flipped: bool,
    ) -> Self {
        return if pressed_up && pressed_right {
            DashDirection::UpRight
        } else if pressed_down && pressed_right {
            DashDirection::DownRight
        } else if pressed_down && pressed_left {
            DashDirection::DownLeft
        } else if pressed_up && pressed_left {
            DashDirection::UpLeft
        } else if pressed_up {
            DashDirection::Up
        } else if pressed_right {
            DashDirection::Right
        } else if pressed_down {
            DashDirection::Down
        } else if pressed_left {
            DashDirection::Left
        } else if is_flipped {
            DashDirection::Left
        } else {
            DashDirection::Right
        };
    }

    pub fn to_vec(self) -> Vec2 {
        match self {
            Self::Up => Vec2::new(0., 1.),
            Self::Down => Vec2::new(0., -1.),
            Self::Right => Vec2::new(1., 0.),
            Self::Left => Vec2::new(-1., 0.),

            Self::UpRight => Vec2::new(1., 1.),
            Self::DownRight => Vec2::new(1., -1.),
            Self::DownLeft => Vec2::new(-1., -1.),
            Self::UpLeft => Vec2::new(-1., 1.),
        }
    }
}
