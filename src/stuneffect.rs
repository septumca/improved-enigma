use bevy::{prelude::*};

use crate::{GameState};


#[derive(Component)]
pub struct StunEffect;

#[derive(Component)]
pub struct Stun(pub Timer);

pub struct StunPlugin;

impl Plugin for StunPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(
                (
                  update_stun,
                ).in_set(OnUpdate(GameState::Playing)));
    }
}

fn update_stun(
    mut commands: Commands,
    timer: Res<Time>,
    mut stun_q: Query<(Entity, &Children, &mut Stun)>,
    stun_effect_q: Query<Entity, (Without<Stun>, With<StunEffect>)>,
) {
    let Ok((entity, children, mut stun)) = stun_q.get_single_mut() else {
        return;
    };
    if stun.0.tick(timer.delta()).finished() {
        commands.entity(entity).remove::<Stun>();
        for &child in children.iter() {
            if stun_effect_q.get(child).is_ok() {
                commands.entity(entity).remove_children(&[child]);
                commands.entity(child).despawn_recursive();
            }
        }
    }
}
