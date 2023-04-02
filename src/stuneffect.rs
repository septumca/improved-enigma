use bevy::{prelude::*};

use crate::{GameState, GameResources, player::{Rotation, LeftSki, RightSki, get_graphics, Player}, Alive, yeti::Yeti, animation::Animation};


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
                  update_wake_up.after(update_stun)
                ).in_set(OnUpdate(GameState::Playing)));
    }
}

pub fn update_stun(
    mut commands: Commands,
    time: Res<Time>,
    mut stun_q: Query<(Entity, &Children, &mut Stun)>,
    stun_effect_q: Query<Entity, (Without<Stun>, With<StunEffect>)>,
) {
    let dt = time.delta();
    for (entity, children, mut stun) in stun_q.iter_mut() {
        if stun.0.tick(dt).finished() {
            commands.entity(entity).remove::<Stun>();
            for &child in children.iter() {
                if stun_effect_q.get(child).is_ok() {
                    commands.entity(entity).remove_children(&[child]);
                    commands.entity(child).despawn_recursive();
                }
            }
        }
    };
}

pub fn update_wake_up(
    game_resources: Res<GameResources>,
    mut woken_up: RemovedComponents<Stun>,
    mut player_q: Query<(&mut Sprite, &Rotation, &Children), (With<Alive>, With<Player>, Without<Yeti>)>,
    mut skis_q: Query<&mut Visibility, (Or<(With<LeftSki>, With<RightSki>)>, Without<Yeti>, Without<Player>)>,
    mut yeti_q: Query<(&mut Sprite, &mut Animation, &mut Yeti), (With<Alive>, Without<Player>)>,
) {
    for entity_woken in woken_up.iter() {
        if let Ok((mut sprite, rotation, children)) = player_q.get_mut(entity_woken) {
            let (sprite_rect, flip_x) = get_graphics(rotation.0, &game_resources);
            sprite.rect = Some(sprite_rect);
            sprite.flip_x = flip_x;
            for &ch in children {
                if let Ok(mut visibility) = skis_q.get_mut(ch) {
                    *visibility = Visibility::Inherited;
                }
            }
        };

        if let Ok((mut sprite, mut animation, mut yeti)) = yeti_q.get_mut(entity_woken) {
            yeti.ignore_collisions.reset();
            animation.set_frames(vec![
                game_resources.yeti_run[0],
                game_resources.yeti_run[1],
                game_resources.yeti_run[0],
                game_resources.yeti_run[2]
            ]);
            if let Some(rect) = animation.get_frame() {
                sprite.rect = Some(rect);
            }
        };
    }
}