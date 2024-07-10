use bevy::{
    app::{Plugin, Startup, Update},
    asset::Assets,
    ecs::{
        entity::Entity,
        query::Added,
        system::{Commands, Query, Res, ResMut, Resource},
        world::World,
    },
    hierarchy::BuildChildren,
    math::{primitives::Sphere, Vec3},
    pbr::{NotShadowCaster, NotShadowReceiver, PbrBundle},
    prelude::{AlphaMode, Color, Handle, Mesh, StandardMaterial},
    render::view::InheritedVisibility,
    transform::components::{GlobalTransform, Transform},
};
use engine::{
    ability::gravity_ball::{GravityBall, GravityBallActivated, GravityBallProps},
    FootOffset,
};

use crate::{draw::ObjectGraphics, in_plane, shapes::HollowPolygon};

pub struct GravityBallPlugin;

impl Plugin for GravityBallPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, (draw, draw_outline));
    }
}

#[derive(Resource)]
pub struct GravityBallAssets {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
    pub outline_mesh: Handle<Mesh>,
    pub outline_material: Handle<StandardMaterial>,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    props: Res<GravityBallProps>,
) {
    let neutrino_ball_material = StandardMaterial {
        base_color: Color::srgba(0.0, 0.0, 0.0, 0.5),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..Default::default()
    };

    let assets = GravityBallAssets {
        mesh: meshes.add(Sphere::new(props.radius)),
        material: materials.add(neutrino_ball_material.clone()),
        outline_mesh: meshes.add(HollowPolygon {
            radius: props.effect_radius,
            thickness: 0.06,
            vertices: 60,
        }),
        outline_material: materials.add(neutrino_ball_material),
    };
    commands.add(|world: &mut World| world.insert_resource(assets));
}

fn draw(
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

fn draw_outline(
    mut commands: Commands,
    assets: Res<GravityBallAssets>,
    query: Query<(Entity, &FootOffset), Added<GravityBallActivated>>,
) {
    for (entity, foot_offset) in &query {
        commands.entity(entity).with_children(|builder| {
            builder.spawn((
                PbrBundle {
                    mesh: assets.outline_mesh.clone(),
                    material: assets.outline_material.clone(),
                    transform: in_plane().with_translation(Vec3::new(
                        0.0,
                        foot_offset.y + 0.01,
                        0.0,
                    )),
                    ..Default::default()
                },
                NotShadowCaster,
                NotShadowReceiver,
            ));
        });
    }
}
