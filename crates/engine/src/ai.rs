use std::cmp::Ordering;

use bevy_ecs::{
    component::Component,
    entity::Entity,
    query::{With, Without},
    system::{Query, Res},
};
use bevy_rapier3d::prelude::Velocity;
use bevy_transform::components::Transform;

use crate::{ability::properties::AbilityProps, face, Faction, Target, To2d};

pub mod charge;
pub mod simple;

#[derive(Component, Default)]
pub struct AiTarget {
    /// Location of the target. This is not necessarily the entity's location,
    /// but more like where the Ai's cursor would be.
    pub loc: Target,
    pub entity: Option<Entity>,
}

/// Note: This updates the ai to aim at its nearest foe. For now, it partially
/// leads based on `gun` speed.
fn update_target_system<T: Faction, Ai: Component>(
    mut ai_q: Query<
        (&mut Transform, &Velocity, &mut AiTarget),
        (With<T>, Without<T::Foe>, With<Ai>),
    >,
    target_q: Query<(Entity, &Transform, &Velocity), (With<T::Foe>, Without<T>)>,
    props: Res<AbilityProps>,
) {
    let shot_speed = props.gun.speed;

    for (mut transform, velocity, mut target) in ai_q.iter_mut() {
        let closest_target = target_q
            .iter()
            .map(|(e, t, v)| (e, t, v, transform.translation.distance(t.translation)))
            .min_by(|(_, _, _, d1), (_, _, _, d2)| d1.partial_cmp(d2).unwrap_or(Ordering::Equal));
        if let Some((target_entity, target_transform, target_velocity, target_distance)) =
            closest_target
        {
            let dt = target_distance / shot_speed;
            let lead = (target_velocity.linvel - velocity.linvel) * dt * 0.5; // Just partially lead for now
            let lead_translation = (target_transform.translation + lead).to_2d();

            face(&mut transform, lead_translation);
            target.loc.0 = lead_translation;
            target.entity = Some(target_entity);
        }
    }
}
