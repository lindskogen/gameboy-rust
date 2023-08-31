pub trait Tick {
    fn tick(&mut self);
}


pub trait Mem {
    fn read_byte(&self, addr: u16) -> u8;
    fn write_byte(&mut self, addr: u16, value: u8);
}
