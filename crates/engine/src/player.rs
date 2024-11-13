use bevy_ecs::component::Component;
use bevy_ecs::system::Commands;
use bevy_rapier3d::prelude::CoefficientCombineRule;
use bevy_rapier3d::prelude::Collider;
use bevy_rapier3d::prelude::ExternalForce;
use bevy_rapier3d::prelude::Friction;
use bevy_rapier3d::prelude::LockedAxes;
use bevy_rapier3d::prelude::RigidBody;
use bevy_rapier3d::prelude::Velocity;
use bevy_transform::components::Transform;

use crate::ability::Abilities;
use crate::collision::TrackCollisionBundle;
use crate::level::InLevel;
use crate::lifecycle::ENERGY_REGEN;
use crate::status_effect::StatusProps;
use crate::Ally;
use crate::Character;
use crate::CharacterMarker;
use crate::Cooldowns;
use crate::Energy;
use crate::Health;
use crate::Kind;
use crate::MassBundle;
use crate::Object;
use crate::Player;
use crate::Shootable;
use crate::Target;
use crate::ABILITY_Y;
use crate::PLAYER_HEIGHT;
use crate::PLAYER_MASS;
use crate::PLAYER_R;

#[derive(Debug, Component)]
pub struct PlayerInfo {
    pub handle: Player,
    pub abilities: Abilities,
}

impl PlayerInfo {
    pub fn spawn_player(&self, commands: &mut Commands) {
        let id = commands
            .spawn((
                Target::default(),
                self.handle,
                Ally,
                Character {
                    health: Health::new(100.0),
                    energy: Energy::new(100.0, ENERGY_REGEN),
                    object: Object {
                        collider: character_collider(PLAYER_R, PLAYER_HEIGHT),
                        foot_offset: (-PLAYER_HEIGHT * 0.5).into(),
                        body: RigidBody::Dynamic,
                        locked_axes: LockedAxes::ROTATION_LOCKED,
                        kind: Kind::Player,
                        transform: Transform::from_xyz(0.0, PLAYER_HEIGHT * 0.5, 0.0).into(),
                        mass: MassBundle::new(PLAYER_MASS),
                        velocity: Velocity::zero(),
                        force: ExternalForce::default(),
                        in_level: InLevel,
                        statuses: StatusProps {
                            thermal_mass: 1.0,
                            capacitance: 1.0,
                        }
                        .into(),
                        collisions: TrackCollisionBundle::off(),
                    },
                    max_speed: Default::default(),
                    friction: Friction {
                        coefficient: 0.0,
                        combine_rule: CoefficientCombineRule::Min,
                    },
                    shootable: Shootable,
                    abilities: self.abilities.clone(),
                    cooldowns: Cooldowns::new(),
                    desired_movement: Default::default(),
                    ability_offset: ((-PLAYER_HEIGHT * 0.5) + ABILITY_Y.y).into(),
                    marker: CharacterMarker,
                },
            ))
            .id();
        tracing::debug!(?id, "Spawning player");
    }
}

pub fn character_collider(radius: f32, height: f32) -> Collider {
    Collider::cylinder(height * 0.5, radius)
}
