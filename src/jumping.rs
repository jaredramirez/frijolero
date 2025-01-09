use bevy::prelude::*;

#[derive(Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct Jumper {
    pub jumping: bool,
}
