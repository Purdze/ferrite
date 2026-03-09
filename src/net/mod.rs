pub mod connection;
pub mod handler;
pub mod sender;

use std::sync::Arc;

use azalea_block::BlockState;
use azalea_core::position::{BlockPos, ChunkPos};
use azalea_inventory::ItemStack;
use azalea_world::heightmap::HeightmapKind;

pub enum NetworkEvent {
    Connected,
    ChunkLoaded {
        pos: ChunkPos,
        data: Arc<Box<[u8]>>,
        heightmaps: Vec<(HeightmapKind, Box<[u64]>)>,
    },
    ChunkUnloaded {
        pos: ChunkPos,
    },
    ChunkCacheCenter {
        x: i32,
        z: i32,
    },
    PlayerPosition {
        x: f64,
        y: f64,
        z: f64,
        yaw: f32,
        pitch: f32,
    },
    PlayerHealth {
        health: f32,
        food: u32,
        saturation: f32,
    },
    InventoryContent {
        items: Vec<ItemStack>,
    },
    InventorySlot {
        index: u16,
        item: ItemStack,
    },
    ChatMessage {
        text: String,
    },
    BlockUpdate {
        pos: BlockPos,
        state: BlockState,
    },
    SectionBlocksUpdate {
        updates: Vec<(BlockPos, BlockState)>,
    },
    Disconnected {
        reason: String,
    },
}
