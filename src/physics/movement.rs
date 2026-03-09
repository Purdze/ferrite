use winit::keyboard::KeyCode;

use super::aabb::Aabb;
use super::collision::resolve_collision;
use crate::player::LocalPlayer;
use crate::window::input::InputState;
use crate::world::chunk::ChunkStore;

const GRAVITY: f32 = 0.08;
const JUMP_VELOCITY: f32 = 0.42;
const VERTICAL_DRAG: f32 = 0.98;
const HORIZONTAL_DRAG: f32 = 0.91;
const BLOCK_FRICTION: f32 = 0.6;
const GROUND_FRICTION: f32 = BLOCK_FRICTION * HORIZONTAL_DRAG;
const GROUND_ACCEL_FACTOR: f32 = 0.216;
const MOVEMENT_SPEED: f32 = 0.1;
const AIR_ACCELERATION: f32 = 0.02;
const INPUT_FRICTION: f32 = 0.98;
const PLAYER_HALF_WIDTH: f32 = 0.3;
const PLAYER_HEIGHT: f32 = 1.8;

pub fn tick(player: &mut LocalPlayer, input: &InputState, chunk_store: &ChunkStore) {
    let (mut forward, mut strafe) = movement_input(input);

    forward *= INPUT_FRICTION;
    strafe *= INPUT_FRICTION;

    if player.on_ground && input.key_pressed(KeyCode::Space) {
        player.velocity.y = JUMP_VELOCITY;
    }

    let accel = if player.on_ground {
        let friction_cubed = GROUND_FRICTION * GROUND_FRICTION * GROUND_FRICTION;
        MOVEMENT_SPEED * (GROUND_ACCEL_FACTOR / friction_cubed)
    } else {
        AIR_ACCELERATION
    };

    let (sin_yaw, cos_yaw) = player.yaw.sin_cos();
    player.velocity.x += (forward * -sin_yaw + strafe * cos_yaw) * accel;
    player.velocity.z += (forward * -cos_yaw + strafe * -sin_yaw) * accel;

    let aabb = Aabb::from_center(player.position, PLAYER_HALF_WIDTH, PLAYER_HEIGHT / 2.0);
    let (resolved, on_ground) = resolve_collision(chunk_store, aabb, player.velocity);

    player.position += resolved;
    player.on_ground = on_ground;

    player.velocity.y -= GRAVITY;
    player.velocity.y *= VERTICAL_DRAG;

    let h_friction = if player.on_ground {
        GROUND_FRICTION
    } else {
        HORIZONTAL_DRAG
    };
    player.velocity.x *= h_friction;
    player.velocity.z *= h_friction;

    if on_ground && player.velocity.y < 0.0 {
        player.velocity.y = 0.0;
    }
}

fn movement_input(input: &InputState) -> (f32, f32) {
    let mut forward: f32 = 0.0;
    let mut strafe: f32 = 0.0;

    if input.key_pressed(KeyCode::KeyW) {
        forward += 1.0;
    }
    if input.key_pressed(KeyCode::KeyS) {
        forward -= 1.0;
    }
    if input.key_pressed(KeyCode::KeyA) {
        strafe -= 1.0;
    }
    if input.key_pressed(KeyCode::KeyD) {
        strafe += 1.0;
    }

    let len_sq = forward * forward + strafe * strafe;
    if len_sq > 1.0 {
        let len = len_sq.sqrt();
        forward /= len;
        strafe /= len;
    }

    (forward, strafe)
}
