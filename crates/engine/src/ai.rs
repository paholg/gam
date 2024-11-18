use std::cmp::Ordering;

use bevy_ecs::bundle::Bundle;
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::query::With;
use bevy_ecs::query::Without;
use bevy_ecs::schedule::IntoSystemConfigs;
use bevy_ecs::schedule::SystemConfigs;
use bevy_ecs::system::Query;
use bevy_ecs::system::Res;
use bevy_rapier3d::prelude::Velocity;
use bevy_transform::components::Transform;

use self::pathfind::HasPath;
use crate::ability::gun::GunProps;
use crate::ability::gun::StandardGun;
use crate::face;
use crate::Faction;
use crate::Target;
use crate::To2d;

pub mod charge;
pub mod pathfind;

pub fn systems() -> SystemConfigs {
    (
        pathfind::poll_pathfinding_system,
        charge::system_set(),
        pathfind::pathfinding_system,
    )
        .chain()
}

pub trait Ai: Component {
    /// A measure of how "smart" this ai is, from 0.0 to 1.0.
    fn intelligence(&self) -> f32;
}

#[derive(Bundle, Default)]
pub struct AiBundle<A: Ai + Default> {
    pub ai: A,
    target: AiTarget,
    path: HasPath,
}

#[derive(Component, Default)]
pub struct AiTarget {
    pub entity: Option<Entity>,
    /// Location of the target. This is not necessarily the entity's location,
    /// but more like where the Ai's cursor would be.
    pub loc: Target,
}

fn target_closest_system<T: Faction, A: Ai>(
    mut ai_q: Query<(&Transform, &mut AiTarget), (With<T>, With<A>)>,
    target_q: Query<(Entity, &Transform), (With<T::Foe>, Without<T>)>,
) {
    for (transform, mut target) in &mut ai_q {
        if target.entity.and_then(|e| target_q.get(e).ok()).is_some() {
            // We already have a target, and it still exists.
            continue;
        }
        let closest_target = target_q
            .iter()
            .map(|(e, t)| (e, transform.translation.distance_squared(t.translation)))
            .min_by(|(_, d1), (_, d2)| d1.partial_cmp(d2).unwrap_or(Ordering::Equal));
        if let Some((entity, _distance)) = closest_target {
            target.entity = Some(entity);
        }
    }
}

/// Note: This updates the ai to aim at its nearest foe. For now, it leads based
/// on gun speed and intelligence factor.
fn update_target_system<T: Faction, A: Ai>(
    mut ai_q: Query<(&mut Transform, &Velocity, &mut AiTarget, &A), (With<T>, Without<T::Foe>)>,
    target_q: Query<(&Transform, &Velocity), (With<T::Foe>, Without<T>)>,
    props: Res<GunProps<StandardGun>>,
) {
    let shot_speed = props.speed;

    for (mut transform, velocity, mut target, ai) in ai_q.iter_mut() {
        let Some((target_transform, target_velocity)) =
            target.entity.and_then(|e| target_q.get(e).ok())
        else {
            continue;
        };

        let dt = transform.translation.distance(target_transform.translation) / shot_speed;

        // Let's use the ai's intelligence factor to determine how much it should lead.
        let lead_factor = ai.intelligence();
        let lead = (target_velocity.linvel - velocity.linvel) * dt * lead_factor;
        let lead_translation = (target_transform.translation + lead).to_2d();

        face(&mut transform, lead_translation);
        target.loc.0 = lead_translation;
    }
}
