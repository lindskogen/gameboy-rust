use self::cpu::ProcessingUnit;
use self::mem::Memory;

mod cpu;
mod mem;

pub struct Core {
    cpu: ProcessingUnit,
}


impl Core {
    pub fn start() {
        let mut core = Core {
            cpu: ProcessingUnit::new(Memory::new()),
        };

        core.cpu.next();

        println!("{:?}", core.cpu);
    }
}