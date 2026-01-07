use bevy::prelude::*;

#[derive(Debug, Default, Component)]
pub struct Group {
    pub order: u32,
}

#[derive(Debug, Component)]
#[require(Group)]
pub struct CurrentGroup;
