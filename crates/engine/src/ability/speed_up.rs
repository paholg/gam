use crate::status_effect::TimeDilation;

use super::properties::SpeedUpProps;

pub fn speed_up(props: &SpeedUpProps, time_dilation: &mut TimeDilation) {
    time_dilation.add_effect(props.amount, props.duration);
}
