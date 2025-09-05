use bevy::{
    app::{Plugin, Startup, Update},
    asset::Assets,
    color::palettes::css::BLACK,
    ecs::{
        entity::Entity,
        query::Added,
        resource::Resource,
        system::{Commands, Query, Res, ResMut},
        world::World,
    },
    math::{primitives::Sphere, Vec3},
    pbr::{MeshMaterial3d, NotShadowCaster, NotShadowReceiver},
    prelude::{
        GlobalTransform, Handle, InheritedVisibility, Mesh, Mesh3d, StandardMaterial, Transform,
        Without,
    },
};
use engine::{
    ability::gravity_ball::{GravityBall, GravityBallGravityField},
    collision::TrackCollisions,
    FootOffset,
};

use super::HasOutline;
use crate::{in_plane, shapes::HollowPolygon};

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
    commands.queue(|world: &mut World| world.insert_resource(assets));
}

fn draw_gravity_ball(
    mut commands: Commands,
    assets: Res<GravityBallAssets>,
    query: Query<Entity, Added<GravityBall>>,
) {
    for entity in query.iter() {
        let Ok(mut ecmds) = commands.get_entity(entity) else {
            continue;
        };
        ecmds.insert((InheritedVisibility::default(),));

        ecmds.with_children(|builder| {
            builder.spawn((
                MeshMaterial3d::from(assets.material.clone_weak()),
                Mesh3d::from(assets.mesh.clone_weak()),
                Transform::IDENTITY,
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
                    MeshMaterial3d::from(assets.outline_material.clone_weak()),
                    Mesh3d::from(assets.outline_mesh.clone()),
                    in_plane().with_translation(offset),
                    NotShadowCaster,
                    NotShadowReceiver,
                ));
            });
    }
}
