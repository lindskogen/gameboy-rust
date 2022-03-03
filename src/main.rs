extern crate bitflags;
extern crate dmg;
use dmg::dmg::Core;

fn main() {
    Core::start(Some("./DMG_ROM.bin"));
}
