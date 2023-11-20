use bevy_ecs::prelude::{Commands, Component, Entity, Query, Res};
use bevy_hierarchy::DespawnRecursiveExt;
use bevy_rapier3d::prelude::RapierContext;

use crate::Health;

#[derive(Clone, Copy, Debug)]
pub enum ExplosionKind {
    FragGrenade,
    HealGrenade,
    SeekerRocket,
}

#[derive(Debug, Component)]
pub struct Explosion {
    pub damage: f32,
    pub kind: ExplosionKind,
}

// Explosions only last one frame.
pub fn explosion_despawn_system(
    mut commands: Commands,
    query: Query<(Entity, &Explosion)>,
    mut health_query: Query<&mut Health>,
    rapier: Res<RapierContext>,
) {
    for (entity, explosion) in &query {
        let targets = rapier
            .intersections_with(entity)
            .filter_map(|(e1, e2, intersecting)| if intersecting { Some((e1, e2)) } else { None })
            .map(|(e1, e2)| if e1 == entity { e2 } else { e1 });
        for target in targets {
            if let Ok(mut health) = health_query.get_mut(target) {
                health.take(explosion.damage);
            }
        }
        commands.entity(entity).despawn_recursive();
    }
}
