//! HUD: снимок игрового состояния и связи, который периодически уходит в React.
//!
//! React перерисовывает HUD сам, но FPS и позиция игрока живут в Bevy. Публикация
//! раз в несколько кадров (~15 Гц) — компромисс: цифры успевают меняться, а
//! встроенный V8 не просыпается на каждый кадр.

use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy_react::prelude::ReactEvents;
use lightyear::{core::timeline::LocalTimeline, prelude::Controlled};

use super::{
    states::{ClientState, SelectedServer},
    ui::HudUpdated,
};

/// Как часто публикуется снимок HUD (в кадрах).
const PUBLISH_EVERY: u32 = 4;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HudTicker>()
            .add_systems(Update, publish_hud.run_if(in_game));
    }
}

fn in_game(state: Res<State<ClientState>>) -> bool {
    matches!(state.get(), ClientState::InGame | ClientState::Paused)
}

#[derive(Resource, Default)]
struct HudTicker(u32);

fn publish_hud(
    mut ticker: ResMut<HudTicker>,
    diagnostics: Res<DiagnosticsStore>,
    timeline: Option<Res<LocalTimeline>>,
    selected: Res<SelectedServer>,
    player: Option<Single<&Transform, With<Controlled>>>,
    events: ReactEvents,
) {
    ticker.0 += 1;
    if !ticker.0.is_multiple_of(PUBLISH_EVERY) {
        return;
    }

    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|fps| fps.smoothed())
        .unwrap_or_default()
        .round() as u32;

    events.send(&HudUpdated {
        server: selected.0.to_string(),
        tick: timeline
            .as_ref()
            .map(|t| t.tick().0 as u32)
            .unwrap_or_default(),
        fps,
        position: player
            .as_ref()
            .map(|transform| transform.translation.to_array()),
    });
}
