use bevy::app::Plugin;
use bevy::app::Startup;
use bevy::app::Update;
use bevy::color::LinearRgba;
use bevy::pbr::MeshMaterial3d;
use bevy::pbr::NotShadowReceiver;
use bevy::prelude::Added;
use bevy::prelude::AlphaMode;
use bevy::prelude::Assets;
use bevy::prelude::BuildChildren;
use bevy::prelude::ChildBuild;
use bevy::prelude::Commands;
use bevy::prelude::Component;
use bevy::prelude::Cylinder;
use bevy::prelude::Entity;
use bevy::prelude::Handle;
use bevy::prelude::InheritedVisibility;
use bevy::prelude::Mesh;
use bevy::prelude::Mesh3d;
use bevy::prelude::Parent;
use bevy::prelude::Query;
use bevy::prelude::Res;
use bevy::prelude::ResMut;
use bevy::prelude::Resource;
use bevy::prelude::StandardMaterial;
use bevy::prelude::Transform;
use bevy::prelude::Vec3;
use bevy::prelude::With;
use bevy::prelude::Without;
use bevy::prelude::World;
use engine::ability::transport::TransportBeam;
use engine::To3d;

use crate::color_gradient::ColorGradient;

pub struct TransportBeamPlugin;
impl Plugin for TransportBeamPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, (draw_transport_system, update_transport_system));
    }
}

#[derive(Resource)]
struct TransportBeamAssets {
    gradient: ColorGradient,
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let gradient = ColorGradient::new([
        (0.0, LinearRgba::new(0.0, 0.0, 2.0, 0.1)),
        (0.5, LinearRgba::new(0.0, 1.0, 2.0, 0.4)),
        (0.8, LinearRgba::new(0.0, 5.0, 5.0, 0.4)),
        (1.0, LinearRgba::new(0.0, 100.0, 100.0, 0.6)),
    ]);
    let base_color = gradient.get(0.0);
    let assets = TransportBeamAssets {
        gradient,
        mesh: meshes.add(Cylinder::new(1.0, 1.0)),
        material: materials.add(StandardMaterial {
            base_color: base_color.into(),
            alpha_mode: AlphaMode::Blend,
            ..Default::default()
        }),
    };

    commands.queue(|world: &mut World| world.insert_resource(assets));
}

#[derive(Component)]
struct TransportReceiverGraphics;

#[derive(Component)]
struct TransportSenderGraphics {
    receiver: Entity,
}

fn draw_transport_system(
    mut commands: Commands,
    assets: Res<TransportBeamAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<(Entity, &TransportBeam, &Transform), Added<TransportBeam>>,
) {
    for (entity, beam, transform) in &query {
        // Clone the material because we're going to mutate it. Probably we
        // could do this better.
        let mat = materials.get(&assets.material).unwrap().clone();
        let material = materials.add(mat);

        commands
            .entity(entity)
            .insert(InheritedVisibility::default());
        commands.entity(entity).with_children(|builder| {
            let receiver = builder
                .spawn((
                    MeshMaterial3d::from(material.clone_weak()),
                    Mesh3d::from(assets.mesh.clone_weak()),
                    Transform::from_scale(Vec3::new(beam.radius, 0.0, beam.radius))
                        .with_translation(beam.destination.to_3d(0.0) - transform.translation),
                    TransportReceiverGraphics,
                    NotShadowReceiver,
                ))
                .id();

            builder.spawn((
                MeshMaterial3d::from(material),
                Mesh3d::from(assets.mesh.clone_weak()),
                Transform::from_scale(Vec3::new(beam.radius, 0.0, beam.radius))
                    .with_translation(Vec3::new(0.0, beam.height, 0.0)),
                TransportSenderGraphics { receiver },
                NotShadowReceiver,
            ));
        });
    }
}

fn update_transport_system(
    assets: Res<TransportBeamAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut sender_q: Query<(
        &Parent,
        &mut Transform,
        &MeshMaterial3d<StandardMaterial>,
        &TransportSenderGraphics,
    )>,
    mut receiver_q: Query<
        &mut Transform,
        (
            With<TransportReceiverGraphics>,
            Without<TransportSenderGraphics>,
        ),
    >,
    parent_q: Query<
        (&Transform, &TransportBeam),
        (
            Without<TransportSenderGraphics>,
            Without<TransportReceiverGraphics>,
        ),
    >,
) {
    for (parent, mut transform, material, sender) in &mut sender_q {
        let Ok((parent_transform, beam)) = parent_q.get(parent.get()) else {
            tracing::warn!("TransportSenderGraphics missing parent");
            continue;
        };

        let frac = 1.0 - beam.activates_in / beam.delay;

        let color = assets.gradient.get(frac);
        materials.get_mut(material).unwrap().base_color = color.into();
        transform.scale.y = frac * beam.height;
        transform.translation.y = beam.height - (frac * beam.height * 0.5);

        let Ok(mut receiver_transform) = receiver_q.get_mut(sender.receiver) else {
            tracing::warn!("Transport sender with no receiver");
            continue;
        };
        receiver_transform.scale.y = frac * beam.height;
        receiver_transform.translation =
            beam.destination.to_3d(frac * beam.height * 0.5) - parent_transform.translation;
    }
}
