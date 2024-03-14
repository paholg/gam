use std::{fmt::Debug, sync::Arc};

use bevy_ecs::{
    entity::Entity,
    system::{Command, Commands, In, ResMut, Resource, System},
    world::World,
};
use bevy_reflect::TypeUuid;
use bevy_utils::{HashMap, Uuid};

use crate::multiplayer::Action;

use self::properties::AbilityProps;

pub mod bullet;
pub mod cooldown;
pub mod grenade;
pub mod gun;
pub mod neutrino_ball;
pub mod properties;
pub mod seeker_rocket;
pub mod speed_up;
pub mod transport;

pub type AbilitySystem = Arc<dyn System<In = Entity, Out = ()>>;

pub struct Ability {
    pub id: Uuid,
    pub name: String,
    pub action: Action,
    pub system: AbilitySystem,
    pub setup_system: AbilitySystem,
    pub secondary: Option<Uuid>,
}

impl Ability {
    pub fn new<S, S2>(
        id: Uuid,
        name: impl Into<String>,
        action: Action,
        system: S,
        setup_system: S2,
    ) -> Self
    where
        S: System<In = Entity, Out = ()>,
        S2: System<In = Entity, Out = ()>,
    {
        Self {
            id,
            name: name.into(),
            action,
            system: Arc::new(system),
            setup_system: Arc::new(setup_system),
            secondary: None,
        }
    }

    pub fn with_secondary(mut self, secondary: Uuid) -> Self {
        self.secondary = Some(secondary);
        self
    }
}

#[derive(Resource, Default)]
pub struct AbilityMap {
    map: HashMap<Uuid, AbilitySystem>,
    registry: HashMap<Action, Vec<Ability>>,
}

impl AbilityMap {
    pub fn register(&mut self, ability: Ability) {
        self.map.insert(ability.id, ability.system);
        self.registry
            .entry(ability.action)
            .or_insert(Vec::new())
            .push(ability);
    }

    // pub fn register_arm<A: ArmAbilityBuilder + TypeUuid>(
    //     &mut self,
    //     builder: A,
    //     name: String,
    //     action: Action,
    // ) {
    //     let val = AbilityInfo {
    //         builder: Box::new(builder) as Box<dyn ArmAbilityBuilder>,
    //         name,
    //         action,
    //     };
    //     self.arm.insert(A::TYPE_UUID, val);
    // }

    pub fn get(&self, key: Uuid) -> AbilitySystem {
        self.map.get(&key).unwrap().clone()
    }

    // pub fn get_arm(
    //     &self,
    //     key: Uuid,
    //     user: Entity,
    //     commands: &mut Commands,
    //     props: &AbilityProps,
    // ) -> Option<(Box<dyn AbilityCommand>, Box<dyn AbilityCommand>)> {
    //     self.arm
    //         .get(&key)
    //         .map(|a| a.builder.build(user, commands, props))
    // }
}

pub trait Side: Debug + Default + Send + Sync + Clone + Copy + 'static {}
#[derive(Debug, Copy, Clone, Default, TypeUuid)]
#[uuid = "fcbdcc90-24d9-44c0-908e-9fc469025590"]
pub struct Left;
#[derive(Debug, Copy, Clone, Default, TypeUuid)]
#[uuid = "5a6c4bdf-e87b-4add-b4f2-c3d34a516063"]
pub struct Right;

impl Side for Left {}
impl Side for Right {}

// pub trait AbilityBuilder: Send + Sync + Debug + 'static {
//     fn build(
//         &self,
//         user: Entity,
//         commands: &mut Commands,
//         props: &AbilityProps,
//     ) -> Box<dyn Ability>;
// }

// pub trait ArmAbilityBuilder: Send + Sync + Debug + 'static {
//     fn build(
//         &self,
//         user: Entity,
//         commands: &mut Commands,
//         props: &AbilityProps,
//     ) -> (Box<dyn Ability>, Box<dyn Ability>);
// }

// pub trait Ability: Command + Send + Sync + Debug + 'static {
//     fn add_command(&self, commands: &mut Commands);
// }

pub fn noop_ability(_: In<Entity>) {}

#[derive(Clone)]
pub struct AbilityCommand {
    user: Entity,
    ability: AbilitySystem,
}

impl AbilityCommand {
    pub fn new(user: Entity, ability: AbilitySystem) -> Self {
        Self { user, ability }
    }
}

impl Command for AbilityCommand {
    fn apply(mut self, world: &mut World) {
        self.ability.run(self.user, world);
    }
}

pub fn setup(mut map: ResMut<AbilityMap>) {
    // map.register(NoAbilityBuilder, "noop".to_owned(), Action::none());
}

// #[derive(Debug)]
// pub struct NoAbility;

// impl Command for NoAbility {
//     fn apply(self, _world: &mut bevy_ecs::world::World) {}
// }

// impl Ability for NoAbility {
//     fn add_command(&self, _commands: &mut Commands) {}
// }

// #[derive(Debug, Default, TypeUuid)]
// #[uuid = "406157e1-38ce-443e-957b-e0dc81284f6d"]
// pub struct NoAbilityBuilder;

// impl AbilityBuilder for NoAbilityBuilder {
//     fn build(
//         &self,
//         _user: Entity,
//         _commands: &mut Commands,
//         _props: &AbilityProps,
//     ) -> Box<dyn Ability> {
//         Box::new(NoAbility)
//     }
// }

// impl ArmAbilityBuilder for NoAbilityBuilder {
//     fn build(
//         &self,
//         _user: Entity,
//         _commands: &mut Commands,
//         _props: &AbilityProps,
//     ) -> (Box<dyn Ability>, Box<dyn Ability>) {
//         (Box::new(NoAbility), Box::new(NoAbility))
//     }
// }

// #[subenum(
//     ArmAbility,
//     ArmSecondaryAbility,
//     ShoulderAbility,
//     LegsAbility,
//     HeadAbility
// )]
// #[derive(
//     Debug,
//     Copy,
//     Clone,
//     Default,
//     Serialize,
//     Deserialize,
//     PartialEq,
//     Eq,
//     EnumIter,
//     Display,
//     Reflect,
//     Hash,
// )]
// pub enum Ability {
//     #[subenum(
//         ArmAbility,
//         ArmSecondaryAbility,
//         ShoulderAbility,
//         LegsAbility,
//         HeadAbility
//     )]
//     #[default]
//     None,
//     #[subenum(ArmAbility)]
//     Gun,
//     #[subenum(ArmSecondaryAbility)]
//     GunReload,
//     #[subenum(ArmAbility)]
//     Shotgun,
//     #[subenum(ArmSecondaryAbility)]
//     ShotgunReload,
//     #[subenum(ArmAbility)]
//     SeekerRocket,
//     #[subenum(ArmSecondaryAbility)]
//     SeekerRocketReload,
//     #[subenum(ShoulderAbility)]
//     FragGrenade,
//     #[subenum(ShoulderAbility)]
//     HealGrenade,
//     #[subenum(ShoulderAbility)]
//     NeutrinoBall,
//     #[subenum(LegsAbility)]
//     Transport,
//     #[subenum(LegsAbility)]
//     SpeedUp,
// }

// impl ArmAbility {
//     pub fn register<S: Side>(&self, user: Entity, commands: &mut Commands, props: &AbilityProps) {
//         match self {
//             ArmAbility::None => (),
//             ArmAbility::Gun => gun::register::<S>(user, commands, props),
//             ArmAbility::Shotgun => shotgun::register::<S>(user, commands, props),
//             ArmAbility::SeekerRocket => seeker_rocket::register::<S>(user, commands, props),
//         }
//     }
//     pub fn secondary(&self) -> &ArmSecondaryAbility {
//         match self {
//             ArmAbility::None => &ArmSecondaryAbility::None,
//             ArmAbility::Gun => &ArmSecondaryAbility::GunReload,
//             ArmAbility::Shotgun => &ArmSecondaryAbility::ShotgunReload,
//             ArmAbility::SeekerRocket => &ArmSecondaryAbility::SeekerRocketReload,
//         }
//     }
// }

// impl Ability {
//     pub fn add_command(&self, user: Entity, commands: &mut Commands) {
//         match self {
//             Ability::None => (),
//             Ability::Gun => ,
//             Ability::GunReload => todo!(),
//             Ability::Shotgun => todo!(),
//             Ability::ShotgunReload => todo!(),
//             Ability::SeekerRocket => todo!(),
//             Ability::SeekerRocketReload => todo!(),
//             Ability::FragGrenade => todo!(),
//             Ability::HealGrenade => todo!(),
//             Ability::NeutrinoBall => todo!(),
//             Ability::Transport => todo!(),
//             Ability::SpeedUp => todo!(),
//         }
//     }
// }

// impl ArmAbility {
//     pub fn secondary(&self) -> ArmSecondaryAbility {
//         match self {
//             ArmAbility::None => ArmSecondaryAbility::None,
//             ArmAbility::Gun => ArmSecondaryAbility::GunReload,
//             ArmAbility::Shotgun => ArmSecondaryAbility::ShotgunReload,
//             ArmAbility::SeekerRocket => ArmSecondaryAbility::SeekerRocketReload,
//         }
//     }
// }

// impl Ability {
//     #[allow(clippy::too_many_arguments)]
//     pub fn fire(
//         &self,
//         commands: &mut Commands,
//         props: &AbilityProps,
//         entity: Entity,
//         energy: &mut Energy,
//         cooldowns: &mut Cooldowns,
//         transform: &Transform,
//         velocity: &Velocity,
//         target: &Target,
//         ability_offset: &AbilityOffset,
//         foot_offset: &FootOffset,
//         time_dilation: &mut TimeDilation,
//     ) -> bool {
//         if cooldowns.is_available(self, time_dilation) && energy.try_use(props.cost(self)) {
//             cooldowns.set(*self, props.cooldown(self));
//         } else {
//             return false;
//         }

//         match self {
//             Ability::None => (),
//             Ability::Gun => gun(
//                 commands,
//                 &props.gun,
//                 transform,
//                 velocity,
//                 entity,
//                 ability_offset,
//             ),
//             Ability::GunReload => todo!(),
//             Ability::Shotgun => shotgun(
//                 commands,
//                 &props.shotgun,
//                 transform,
//                 velocity,
//                 entity,
//                 ability_offset,
//             ),
//             Ability::ShotgunReload => todo!(),
//             Ability::FragGrenade => grenade(
//                 commands,
//                 &props.frag_grenade,
//                 transform,
//                 entity,
//                 target,
//                 ability_offset,
//             ),
//             Ability::HealGrenade => grenade(
//                 commands,
//                 &props.heal_grenade,
//                 transform,
//                 entity,
//                 target,
//                 ability_offset,
//             ),
//             Ability::SeekerRocket => seeker_rocket(
//                 commands,
//                 &props.seeker_rocket,
//                 transform,
//                 velocity,
//                 entity,
//                 ability_offset,
//             ),
//             Ability::SeekerRocketReload => todo!(),
//             Ability::NeutrinoBall => neutrino_ball(
//                 commands,
//                 &props.neutrino_ball,
//                 transform,
//                 velocity,
//                 foot_offset,
//             ),
//             Ability::Transport => transport(commands, entity, &props.transport, transform, target),
//             Ability::SpeedUp => speed_up(&props.speed_up, time_dilation),
//         }
//         true
//     }
// }

// fn shotgun(
//     commands: &mut Commands,
//     props: &ShotgunProps,
//     transform: &Transform,
//     velocity: &Velocity,
//     shooter: Entity,
//     ability_offset: &AbilityOffset,
// ) {
//     for i in 0..props.n_pellets {
//         let idx = i as f32;
//         let n_pellets = props.n_pellets as f32;
//         let relative_angle = (n_pellets * 0.5 - idx) / n_pellets * props.spread;
//         let relative_angle = Quat::from_rotation_z(relative_angle);
//         let dir = (transform.rotation * relative_angle) * FORWARD;
//         let position =
//             transform.translation + dir * (PLAYER_R + props.radius * 2.0) + ability_offset.to_vec();
//         let velocity = dir * props.speed + velocity.linvel;
//         BulletSpawner {
//             position,
//             velocity,
//             radius: props.radius,
//             mass: props.mass,
//             bullet: Bullet {
//                 shooter,
//                 expires_in: props.duration,
//                 damage: props.damage,
//             },
//             health: Health::new(props.bullet_health),
//         }
//         .spawn(commands);
//     }
// }
