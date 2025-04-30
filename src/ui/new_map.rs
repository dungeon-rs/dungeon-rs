use crate::{ToolbarAction, dialog};
use bevy::app::Update;
use bevy::{
    app::App,
    ecs::children,
    prelude::SpawnRelated,
    prelude::{Commands, EventReader, Plugin},
};

pub struct NewMapPlugin;

impl Plugin for NewMapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_file_cmds);
    }
}

fn handle_file_cmds(mut commands: Commands, mut reader: EventReader<ToolbarAction>) {
    if !reader.read().any(|event| match event {
        ToolbarAction::New => true,
        _ => false,
    }) {
        return;
    }

    commands.spawn(dialog("New map", children![]));
}
