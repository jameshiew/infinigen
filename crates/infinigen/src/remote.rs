use bevy::prelude::*;
use bevy::remote;

pub struct RemotePlugin;

impl Plugin for RemotePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            remote::RemotePlugin::default(),
            remote::http::RemoteHttpPlugin::default(),
        ));
    }
}
