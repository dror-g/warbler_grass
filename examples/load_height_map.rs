use bevy::{prelude::*, render::primitives::Aabb};
use warblersneeds::{
    warblers_plugin::WarblersPlugin, WarblersBundle, grass_spawner::GrassSpawner, height_map::HeightMap,
};
mod helper;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(WarblersPlugin)
        .add_plugin(helper::SimpleCamera)
        .add_startup_system(setup_grass)
        .run();
}
fn setup_grass(mut commands: Commands, asset_server: Res<AssetServer>) {
    let height_map = asset_server.load("grass_height_map.png");
    let positions = (0..10_000).into_iter()
        .map(|i| (i / 100, i % 100))
        .map(|(x,z)| Vec3::new(x as f32, 0.,z as f32))
        .collect();
    let height_map = HeightMap {
        height_map,
        aabb: Aabb::from_min_max(Vec3::new(0.,0.,0.), Vec3::new(11.,0.,11.)),
    };
    let grass_spawner = GrassSpawner::new().with_positions(positions).with_height_map(height_map);
    commands.spawn((WarblersBundle { grass_spawner, ..default() },));
}
