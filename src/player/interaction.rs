use azalea_block::BlockState;
use azalea_core::direction::Direction;
use azalea_core::position::BlockPos;
use azalea_protocol::packets::game::s_interact::InteractionHand;
use azalea_protocol::packets::game::s_player_action::{Action, ServerboundPlayerAction};
use azalea_protocol::packets::game::s_use_item_on::{BlockHit, ServerboundUseItemOn};
use azalea_protocol::packets::game::ServerboundGamePacket;
use glam::Vec3;

use crate::net::sender::PacketSender;
use crate::window::input::InputState;
use crate::world::chunk::ChunkStore;

const REACH: f32 = 4.5;
const STEP: f32 = 0.01;

#[derive(Debug, Clone, Copy)]
pub struct HitResult {
    pub block_pos: BlockPos,
    pub face: Direction,
    pub hit_point: Vec3,
}

pub struct InteractionState {
    pub target: Option<HitResult>,
    seq: u32,
}

impl InteractionState {
    pub fn new() -> Self {
        Self {
            target: None,
            seq: 0,
        }
    }

    pub fn update_target(&mut self, eye: Vec3, yaw: f32, pitch: f32, chunks: &ChunkStore) {
        let dir = look_direction(yaw, pitch);
        self.target = raycast(eye, dir, REACH, chunks);
    }

    pub fn tick(
        &mut self,
        input: &InputState,
        chunks: &ChunkStore,
        sender: Option<&PacketSender>,
    ) -> Vec<azalea_core::position::ChunkPos> {
        let mut dirty_chunks = Vec::new();

        if !input.is_cursor_captured() {
            return dirty_chunks;
        }

        let Some(hit) = self.target else {
            if input.left_just_pressed() {
                if let Some(sender) = sender {
                    send_swing(sender);
                }
            }
            return dirty_chunks;
        };

        if input.left_just_pressed() {
            self.seq += 1;
            if let Some(sender) = sender {
                send_swing(sender);
                sender.send(ServerboundGamePacket::PlayerAction(ServerboundPlayerAction {
                    action: Action::StartDestroyBlock,
                    pos: hit.block_pos,
                    direction: hit.face,
                    seq: self.seq,
                }));
                sender.send(ServerboundGamePacket::PlayerAction(ServerboundPlayerAction {
                    action: Action::StopDestroyBlock,
                    pos: hit.block_pos,
                    direction: hit.face,
                    seq: self.seq,
                }));
            }
            chunks.set_block_state(hit.block_pos.x, hit.block_pos.y, hit.block_pos.z, BlockState::AIR);
            mark_dirty(&hit.block_pos, &mut dirty_chunks);
        }

        if input.right_just_pressed() {
            self.seq += 1;
            if let Some(sender) = sender {
                sender.send(ServerboundGamePacket::UseItemOn(ServerboundUseItemOn {
                    hand: InteractionHand::MainHand,
                    block_hit: BlockHit {
                        block_pos: hit.block_pos,
                        direction: hit.face,
                        location: azalea_core::position::Vec3 {
                            x: hit.hit_point.x as f64,
                            y: hit.hit_point.y as f64,
                            z: hit.hit_point.z as f64,
                        },
                        inside: false,
                        world_border: false,
                    },
                    seq: self.seq,
                }));
                send_swing(sender);
            }
        }

        dirty_chunks
    }
}

fn mark_dirty(
    pos: &BlockPos,
    dirty: &mut Vec<azalea_core::position::ChunkPos>,
) {
    let chunk_pos = azalea_core::position::ChunkPos::new(
        pos.x.div_euclid(16),
        pos.z.div_euclid(16),
    );
    if !dirty.contains(&chunk_pos) {
        dirty.push(chunk_pos);
    }

    let local_x = pos.x.rem_euclid(16);
    let local_z = pos.z.rem_euclid(16);
    let neighbors = [
        (local_x == 0, -1, 0),
        (local_x == 15, 1, 0),
        (local_z == 0, 0, -1),
        (local_z == 15, 0, 1),
    ];
    for (on_edge, dx, dz) in neighbors {
        if on_edge {
            let np = azalea_core::position::ChunkPos::new(chunk_pos.x + dx, chunk_pos.z + dz);
            if !dirty.contains(&np) {
                dirty.push(np);
            }
        }
    }
}

fn look_direction(yaw: f32, pitch: f32) -> Vec3 {
    Vec3::new(
        -yaw.sin() * pitch.cos(),
        pitch.sin(),
        -yaw.cos() * pitch.cos(),
    )
}

fn raycast(origin: Vec3, dir: Vec3, max_dist: f32, chunks: &ChunkStore) -> Option<HitResult> {
    let mut t = 0.0;
    let mut prev_block = BlockPos { x: i32::MAX, y: i32::MAX, z: i32::MAX };

    while t <= max_dist {
        let point = origin + dir * t;
        let bx = point.x.floor() as i32;
        let by = point.y.floor() as i32;
        let bz = point.z.floor() as i32;
        let block_pos = BlockPos { x: bx, y: by, z: bz };

        if block_pos != prev_block {
            let state = chunks.get_block_state(bx, by, bz);
            if !state.is_air() {
                let face = hit_face(origin, dir, &block_pos);
                return Some(HitResult {
                    block_pos,
                    face,
                    hit_point: point,
                });
            }
            prev_block = block_pos;
        }

        t += STEP;
    }
    None
}

fn hit_face(origin: Vec3, dir: Vec3, pos: &BlockPos) -> Direction {
    let min = Vec3::new(pos.x as f32, pos.y as f32, pos.z as f32);
    let max = min + Vec3::ONE;

    let mut best_t = f32::MAX;
    let mut best_face = Direction::Up;

    let faces: [(f32, f32, f32, Direction); 6] = [
        (min.x, dir.x, origin.x, Direction::West),
        (max.x, dir.x, origin.x, Direction::East),
        (min.y, dir.y, origin.y, Direction::Down),
        (max.y, dir.y, origin.y, Direction::Up),
        (min.z, dir.z, origin.z, Direction::North),
        (max.z, dir.z, origin.z, Direction::South),
    ];

    for &(plane, d_comp, o_comp, face) in &faces {
        if d_comp.abs() < 1e-8 {
            continue;
        }
        let t = (plane - o_comp) / d_comp;
        if t < 0.0 || t >= best_t {
            continue;
        }
        let hit = origin + dir * t;
        let (c1, c2, c1_min, c1_max, c2_min, c2_max) = match face {
            Direction::West | Direction::East => (hit.y, hit.z, min.y, max.y, min.z, max.z),
            Direction::Down | Direction::Up => (hit.x, hit.z, min.x, max.x, min.z, max.z),
            Direction::North | Direction::South => (hit.x, hit.y, min.x, max.x, min.y, max.y),
        };
        if c1 >= c1_min && c1 <= c1_max && c2 >= c2_min && c2 <= c2_max {
            best_t = t;
            best_face = face;
        }
    }

    best_face
}

fn send_swing(sender: &PacketSender) {
    use azalea_protocol::packets::game::s_swing::ServerboundSwing;
    sender.send(ServerboundGamePacket::Swing(ServerboundSwing {
        hand: InteractionHand::MainHand,
    }));
}
