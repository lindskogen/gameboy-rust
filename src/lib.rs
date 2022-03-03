extern crate bitflags;
pub mod dmg;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let _d = super::dmg::Core::start(None);
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn it_can_load_bootrom() {
        let _d = super::dmg::Core::start(Some("./DMG_ROM.bin"));
        assert_eq!(2 + 2, 4);
    }
}
