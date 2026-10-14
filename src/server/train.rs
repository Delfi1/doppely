use std::ops::Rem;

use bevy::prelude::*;
use dfdx::optim::*;
use dfdx::prelude::*;
use doppely::PlayerAction;
use leafwing_input_manager::action_state::ActionState;
use lightyear::core::timeline::LocalTimeline;

#[cfg(feature = "cuda")]
type Device = Cuda;

#[cfg(not(feature = "cuda"))]
type Device = Cpu;

type Model = (Linear<84, 128>, ReLU, Linear<128, 64>);

type NpcModel = <Model as BuildOnDevice<Device, f32>>::Built;

#[derive(Resource, Default, Deref, DerefMut)]
pub struct ComputeDevice(pub Device);

unsafe impl Sync for ComputeDevice {}
unsafe impl Send for ComputeDevice {}

unsafe impl Sync for Npc {}
unsafe impl Send for Npc {}

#[derive(Resource)]
pub struct Npc {
    pub target: Option<Entity>,
    pub model: NpcModel,
    pub optimizer: Adam<NpcModel, f32, Device>,
}

impl FromWorld for Npc {
    fn from_world(world: &mut World) -> Self {
        let dev = world.get_resource_or_init::<ComputeDevice>();

        let model = NpcModel::build(&dev);
        let optimizer = Adam::new(
            &model,
            AdamConfig {
                lr: 1e-3,
                betas: [0.9, 0.999],
                eps: 1e-8,
                weight_decay: Some(WeightDecay::Decoupled(1e-2)),
            },
        );

        Self {
            target: None,
            model,
            optimizer,
        }
    }
}

pub struct TrainPlugin;

impl Plugin for TrainPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Npc>();
        app.add_systems(FixedUpdate, take_batch);
    }
}

// todo: Create data batches for model training
pub fn take_batch(
    time: Res<LocalTimeline>,
    npc: Res<Npc>,
    query: Query<&ActionState<PlayerAction>>,
) {
    if !(time.tick().rem(64) == 0) {
        return;
    }

    let Some(target) = npc.target else {
        return;
    };

    let Some(actions) = query.get(target.entity()).ok() else {
        return;
    };

    let mut buf = [0.0; 64];

    let m = actions.axis_pair(&PlayerAction::Move);
    buf[0] = m.x;
    buf[1] = m.y;
    let r = actions.axis_pair(&PlayerAction::Rotation);
    buf[2] = r.x;
    buf[3] = r.y;

    bevy::log::info!("Batch Buffer: {:?}", buf);
}
