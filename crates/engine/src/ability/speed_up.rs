use super::properties::SpeedUpProps;
use crate::status_effect::TimeDilation;

pub fn speed_up(props: &SpeedUpProps, time_dilation: &mut TimeDilation) {
    time_dilation.add_effect(props.amount, props.duration);
}
