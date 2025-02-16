use bevy::app::Plugin;
use bevy::app::Startup;
use bevy::app::Update;
use bevy::asset::Assets;
use bevy::asset::Handle;
use bevy::color::LinearRgba;
use bevy::pbr::MeshMaterial3d;
use bevy::pbr::StandardMaterial;
use bevy::prelude::Added;
use bevy::prelude::BuildChildren;
use bevy::prelude::Capsule3d;
use bevy::prelude::ChildBuild;
use bevy::prelude::Commands;
use bevy::prelude::Entity;
use bevy::prelude::InheritedVisibility;
use bevy::prelude::Mesh;
use bevy::prelude::Mesh3d;
use bevy::prelude::Query;
use bevy::prelude::Res;
use bevy::prelude::ResMut;
use bevy::prelude::Resource;
use bevy::prelude::Vec2;
use bevy::prelude::World;
use engine::ability::rocket::Rocket;
use engine::Energy;

use crate::bar::Bar;
use crate::color_gradient::ColorGradient;
use crate::draw::explosion::ExplosionAssets;
use crate::in_plane;

pub struct RocketPlugin;

#[derive(Resource)]
pub struct RocketAssets {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    pub explosion: ExplosionAssets,
}

impl Plugin for RocketPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, draw_rocket_system);
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let seeker_rocket_material = StandardMaterial {
        emissive: LinearRgba::rgb(20.0, 1.0, 20.0),
        ..Default::default()
    };

    let assets = RocketAssets {
        mesh: meshes.add(Capsule3d::new(1.0, 1.0)),
        material: materials.add(seeker_rocket_material.clone()),
        explosion: ExplosionAssets::new(
            ColorGradient::new([
                (0.0, LinearRgba::new(50.0, 1.0, 50.0, 0.2)),
                (0.5, LinearRgba::new(100.0, 1.0, 100.0, 0.2)),
                (0.8, LinearRgba::new(2.0, 2.0, 2.0, 0.2)),
                (1.0, LinearRgba::new(0.0, 0.0, 0.0, 0.1)),
            ]),
            &mut meshes,
            &mut materials,
        ),
    };
    commands.queue(|world: &mut World| world.insert_resource(assets));
}

pub fn draw_rocket_system(
    mut commands: Commands,
    assets: Res<RocketAssets>,
    query: Query<Entity, Added<Rocket>>,
) {
    for entity in query.iter() {
        let Some(mut ecmds) = commands.get_entity(entity) else {
            continue;
        };
        ecmds.insert((InheritedVisibility::default(),));

        ecmds.with_children(|builder| {
            builder.spawn((
                MeshMaterial3d::from(assets.material.clone_weak()),
                Mesh3d::from(assets.mesh.clone_weak()),
                Bar::<Energy>::new(0.2, Vec2::new(0.15, 0.08)),
                in_plane(),
            ));
        });
    }
}
