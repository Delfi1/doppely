use bevy::prelude::*;
use dfdx::optim::*;
use dfdx::prelude::*;
use doppely::PlayerId;

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

#[derive(Component)]
pub struct Npc {
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

        Self { model, optimizer }
    }
}

pub struct TrainPlugin;

impl Plugin for TrainPlugin {
    fn build(&self, app: &mut App) {}
}

#[derive(Resource)]
pub struct SelectedPlayer(pub Entity);

// todo: Create data batches for model training
pub fn take_batch(player: Option<Res<SelectedPlayer>>, query: Query<Entity, With<PlayerId>>) {
    let Some(player) = player else { return };
    let Some(data) = query.get(player.0).ok() else {
        return;
    };
}
