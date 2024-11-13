use bevy::pbr::NotShadowReceiver;
use bevy::prelude::Added;
use bevy::prelude::Assets;
use bevy::prelude::BuildChildren;
use bevy::prelude::Commands;
use bevy::prelude::Component;
use bevy::prelude::Entity;
use bevy::prelude::GlobalTransform;
use bevy::prelude::Handle;
use bevy::prelude::InheritedVisibility;
use bevy::prelude::Parent;
use bevy::prelude::Query;
use bevy::prelude::Res;
use bevy::prelude::ResMut;
use bevy::prelude::StandardMaterial;
use bevy::prelude::Transform;
use bevy::prelude::Vec3;
use bevy::prelude::With;
use bevy::prelude::Without;
use engine::ability::transport::TransportBeam;
use engine::To3d;

use super::ObjectGraphics;
use crate::asset_handler::AssetHandler;

#[derive(Component)]
pub struct TransportSenderGraphics {
    receiver: Entity,
}

#[derive(Component)]
pub struct TransportReceiverGraphics;

pub fn draw_transport_system(
    mut commands: Commands,
    assets: Res<AssetHandler>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<(Entity, &TransportBeam, &Transform), Added<TransportBeam>>,
) {
    for (entity, beam, transform) in &query {
        // Clone the material because we're going to mutate it. Probably we
        // could do this better.
        let mat = materials.get(&assets.transport.material).unwrap().clone();
        let material = materials.add(mat);

        commands
            .entity(entity)
            .insert(InheritedVisibility::default());
        commands.entity(entity).with_children(|builder| {
            let receiver = builder
                .spawn((
                    ObjectGraphics {
                        material: material.clone(),
                        mesh: assets.transport.mesh.clone(),
                        ..Default::default()
                    },
                    Transform::from_scale(Vec3::new(beam.radius, 0.0, beam.radius))
                        .with_translation(beam.destination.to_3d(0.0) - transform.translation),
                    GlobalTransform::default(),
                    TransportReceiverGraphics,
                    NotShadowReceiver,
                ))
                .id();

            builder.spawn((
                ObjectGraphics {
                    material,
                    mesh: assets.transport.mesh.clone(),
                    ..Default::default()
                },
                Transform::from_scale(Vec3::new(beam.radius, 0.0, beam.radius))
                    .with_translation(Vec3::new(0.0, beam.height, 0.0)),
                GlobalTransform::default(),
                TransportSenderGraphics { receiver },
                NotShadowReceiver,
            ));
        });
    }
}

pub fn update_transport_system(
    assets: Res<AssetHandler>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut sender_q: Query<(
        &Parent,
        &mut Transform,
        &Handle<StandardMaterial>,
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

        let color = assets.transport.gradient.get(frac);
        materials.get_mut(material).unwrap().base_color = color;
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
