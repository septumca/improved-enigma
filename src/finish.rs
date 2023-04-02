use bevy::{prelude::*};

use crate::{
    GameState,
    player::{
        Player, CompletedRace, Slowdown,
    },
    despawn,
};


#[derive(Component)]
pub struct Finish(pub f32);

pub struct FinishPlugin;

impl Plugin for FinishPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system(despawn::<Finish>.in_schedule(OnExit(GameState::GameOver)))
            .add_systems(
                (
                    check_finish_crossed,
                ).in_set(OnUpdate(GameState::Playing))
            );
    }
}

fn check_finish_crossed(
    mut commands: Commands,
    finish_q: Query<&Finish>,
    player_q: Query<(Entity, &Transform), (With<Player>, Without<Slowdown>, Without<CompletedRace>)>,
) {
    let Ok(finish) = finish_q.get_single() else {
        return;
    };
    let Ok((entity, transform)) = player_q.get_single() else {
        return;
    };

    if transform.translation.y < finish.0 {
        commands.entity(entity).insert(Slowdown(Timer::from_seconds(0.2, TimerMode::Once)));
        commands.entity(entity).insert(CompletedRace);
    }
}
