use bevy::prelude::{NonSendMut, Query, Res, Vec2, With, Without};
use bevy_learn::{reinforce::ReinforceTrainer, Trainer};
use bevy_rapier2d::prelude::{ExternalImpulse, Velocity};

use crate::{Ai, Ally, Enemy, Health, MaxSpeed, PLANE_SIZE};

use super::{Action, AiState};

pub const ACTIONS: [Action; 4] = [
    // Action::Nothing,
    Action::Move(Vec2::new(-1.0, 0.0)),
    Action::Move(Vec2::new(1.0, 0.0)),
    Action::Move(Vec2::new(0.0, -1.0)),
    Action::Move(Vec2::new(0.0, 1.0)),
];

pub fn ai_act(
    mut trainer: NonSendMut<ReinforceTrainer>,
    ai_state: Res<AiState>,
    ally_q: Query<&Health, (With<Ai>, With<Ally>, Without<Enemy>)>,
    mut enemy_q: Query<
        (&Health, &mut Velocity, &MaxSpeed, &mut ExternalImpulse),
        (With<Ai>, With<Enemy>, Without<Ally>),
    >,
) {
    let a_health = match ally_q.get_single() {
        Ok(a) => a,
        Err(_) => return,
    };
    let e = match enemy_q.get_single_mut() {
        Ok(e) => e,
        Err(_) => return,
    };
    let (e_health, mut e_vel, e_max, mut e_impulse) = e;
    let action_id = trainer.pick_action();
    let action = &ACTIONS[action_id as usize];

    match action {
        Action::Nothing => (),
        Action::Move(imp) => {
            e_impulse.impulse = *imp * e_max.impulse;
            e_vel.linvel = e_vel.linvel.clamp_length_max(e_max.max_speed);
        }
    }

    let reward = (ai_state.enemy_dmg_done - ai_state.ally_dmg_done) / e_health.max;

    let obs = &[
        e_health.cur / e_health.max,
        ai_state.enemy_location.x / PLANE_SIZE,
        ai_state.enemy_location.y / PLANE_SIZE,
        a_health.cur / a_health.max,
        ai_state.ally_location.x / PLANE_SIZE,
        ai_state.ally_location.y / PLANE_SIZE,
    ];

    trainer.train(obs, reward);
}
