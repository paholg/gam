use std::cmp::Ordering;

use bevy_ecs::{
    component::Component,
    query::{ReadOnlyWorldQuery, With, Without},
    system::{Query, Res},
};
use bevy_rapier3d::prelude::Velocity;
use bevy_transform::components::Transform;

use crate::{ability::properties::AbilityProps, face, Ally, Enemy, To2d};

pub mod simple;

fn point_to_closest<T: ReadOnlyWorldQuery, U: ReadOnlyWorldQuery>(
    mut query: Query<(&mut Transform, &Velocity), T>,
    targets: Query<(&Transform, &Velocity), U>,
    shot_speed: f32,
) {
    for (mut transform, velocity) in query.iter_mut() {
        let closest_target = targets
            .iter()
            .map(|(t, v)| (t, v, transform.translation.distance(t.translation)))
            .min_by(|(_, _, d1), (_, _, d2)| d1.partial_cmp(d2).unwrap_or(Ordering::Equal));
        if let Some((trans, vel, dist)) = closest_target {
            let dt = dist / shot_speed;
            let lead = (vel.linvel - velocity.linvel) * dt * 0.5; // Just partially lead for now
            let lead_translation = trans.translation + lead;

            face(&mut transform, lead_translation.to_2d());
        }
    }
}

fn update_enemy_orientation<T: Component>(
    ally_query: Query<(&Transform, &Velocity), (With<Ally>, Without<Enemy>)>,
    enemy_query: Query<(&mut Transform, &Velocity), (With<Enemy>, With<T>, Without<Ally>)>,
    props: Res<AbilityProps>,
) {
    let shot_speed = props.gun.speed;
    point_to_closest(enemy_query, ally_query, shot_speed);
}

fn update_ally_orientation<T: Component>(
    enemy_query: Query<(&Transform, &Velocity), (With<Enemy>, Without<Ally>)>,
    ally_query: Query<(&mut Transform, &Velocity), (With<Ally>, With<T>, Without<Enemy>)>,
    props: Res<AbilityProps>,
) {
    let shot_speed = props.gun.speed;
    point_to_closest(ally_query, enemy_query, shot_speed);
}
