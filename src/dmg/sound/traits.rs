

pub trait Tick {
    fn tick(&mut self);
}


pub trait Mem {
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, value: u8);
}
