use std::fs::File;

use crate::dmg::core::Core;

pub fn restore_state() -> Option<Core> {
    let mut f = File::open("state.bin").ok()?;
    serde_cbor::from_reader(&mut f).ok()
}

pub fn save_state(core: &Core) -> serde_cbor::Result<()> {
    let mut f = File::create("state.bin")?;
    serde_cbor::to_writer(&mut f, &core)
}


