use bevy::app::Plugin;
use bevy::app::Startup;
use bevy::app::Update;
use bevy::asset::Assets;
use bevy::color::palettes::css::BLACK;
use bevy::ecs::entity::Entity;
use bevy::ecs::query::Added;
use bevy::ecs::system::Commands;
use bevy::ecs::system::Query;
use bevy::ecs::system::Res;
use bevy::ecs::system::ResMut;
use bevy::ecs::system::Resource;
use bevy::ecs::world::World;
use bevy::math::primitives::Sphere;
use bevy::math::Vec3;
use bevy::pbr::NotShadowCaster;
use bevy::pbr::NotShadowReceiver;
use bevy::pbr::PbrBundle;
use bevy::prelude::BuildChildren;
use bevy::prelude::GlobalTransform;
use bevy::prelude::Handle;
use bevy::prelude::InheritedVisibility;
use bevy::prelude::Mesh;
use bevy::prelude::StandardMaterial;
use bevy::prelude::Transform;
use bevy::prelude::Without;
use engine::ability::gravity_ball::GravityBall;
use engine::ability::gravity_ball::GravityBallGravityField;
use engine::collision::TrackCollisions;
use engine::FootOffset;

use super::HasOutline;
use crate::draw::ObjectGraphics;
use crate::in_plane;
use crate::shapes::HollowPolygon;

pub struct GravityBallPlugin;

#[derive(Resource)]
struct GravityBallAssets {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    outline_mesh: Handle<Mesh>,
    outline_material: Handle<StandardMaterial>,
}

impl Plugin for GravityBallPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, (draw_gravity_ball, draw_gravity_ball_outline));
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let material = StandardMaterial {
        base_color: BLACK.into(),
        unlit: true,
        ..Default::default()
    };

    let assets = GravityBallAssets {
        mesh: meshes.add(Sphere::new(1.0)),
        material: materials.add(material.clone()),
        outline_mesh: meshes.add(HollowPolygon {
            radius: 1.0,
            // TODO: Currently, the thickness scales with size. We should think of a way to make it
            // scale-independent.
            thickness: 0.02,
            vertices: 60,
        }),
        outline_material: materials.add(material),
    };
    commands.add(|world: &mut World| world.insert_resource(assets));
}

fn draw_gravity_ball(
    mut commands: Commands,
    assets: Res<GravityBallAssets>,
    query: Query<Entity, Added<GravityBall>>,
) {
    for entity in query.iter() {
        let Some(mut ecmds) = commands.get_entity(entity) else {
            continue;
        };
        ecmds.insert((InheritedVisibility::default(),));

        ecmds.with_children(|builder| {
            builder.spawn((
                ObjectGraphics {
                    material: assets.material.clone(),
                    mesh: assets.mesh.clone(),
                    ..Default::default()
                },
                Transform::IDENTITY,
                GlobalTransform::default(),
            ));
        });
    }
}

fn draw_gravity_ball_outline(
    mut commands: Commands,
    assets: Res<GravityBallAssets>,
    query: Query<
        (Entity, &FootOffset, &GlobalTransform),
        (
            Added<GravityBallGravityField>,
            Without<HasOutline>,
            Added<TrackCollisions>,
        ),
    >,
) {
    for (entity, foot_offset, global_transform) in &query {
        let scale = global_transform.compute_transform().scale;
        let offset = Vec3::Y * (foot_offset.y / scale.y);
        commands
            .entity(entity)
            .insert(InheritedVisibility::default());
        commands
            .entity(entity)
            .insert(HasOutline)
            .with_children(|builder| {
                builder.spawn((
                    PbrBundle {
                        mesh: assets.outline_mesh.clone(),
                        material: assets.outline_material.clone(),
                        transform: in_plane().with_translation(offset),
                        ..Default::default()
                    },
                    NotShadowCaster,
                    NotShadowReceiver,
                ));
            });
    }
}
