use bevy::reflect::Reflect;
use leafwing_input_manager::Actionlike;

#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
pub enum PlatformerAction {
    Right,
    Left,
    Down,
    Up,
    Jump,
}
