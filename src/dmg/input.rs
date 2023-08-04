use bit_field::BitField;
use bitflags::bitflags;

bitflags! {
    pub struct JoypadInput: u8 {
        const DOWN    = 1 << 0;
        const LEFT    = 1 << 1;
        const UP      = 1 << 2;
        const RIGHT   = 1 << 3;
        const START   = 1 << 4;
        const SELECT  = 1 << 5;
        const A       = 1 << 6;
        const B       = 1 << 7;
    }
}

bitflags! {
    pub struct JoypadOutput: u8 {
        const RIGHT_OR_A    = 1 << 0;
        const LEFT_OR_B     = 1 << 1;
        const UP_OR_SELECT  = 1 << 2;
        const DOWN_OR_START = 1 << 3;
    }
}

#[derive(Eq, PartialEq)]
enum JoypadMode {
    Action,
    Direction,
}

pub struct Joypad {
    mode: JoypadMode,
    input: JoypadInput,
}

impl Default for Joypad {
    fn default() -> Self {
        Self {
            mode: JoypadMode::Action,
            input: JoypadInput::empty(),
        }
    }
}

impl Joypad {
    pub fn write(&mut self, value: u8) {
        let set_direction = value.get_bit(4) == false;
        let set_action = value.get_bit(5) == false;

        if set_direction {
            self.mode = JoypadMode::Direction;
        } else if set_action {
            self.mode = JoypadMode::Action;
        }
    }


    pub fn update(&mut self, input: JoypadInput) {
        self.input = input;
    }

    pub fn read(&self) -> u8 {
        let mut output = JoypadOutput::all();
        if self.mode == JoypadMode::Action {
            if self.input.contains(JoypadInput::START) {
                output.remove(JoypadOutput::DOWN_OR_START);
            }
            if self.input.contains(JoypadInput::SELECT) {
                output.remove(JoypadOutput::UP_OR_SELECT);
            }
            if self.input.contains(JoypadInput::A) {
                output.remove(JoypadOutput::RIGHT_OR_A);
            }
            if self.input.contains(JoypadInput::B) {
                output.remove(JoypadOutput::LEFT_OR_B);
            }
        } else {
            if self.input.contains(JoypadInput::DOWN) {
                output.remove(JoypadOutput::DOWN_OR_START);
            }
            if self.input.contains(JoypadInput::UP) {
                output.remove(JoypadOutput::UP_OR_SELECT);
            }
            if self.input.contains(JoypadInput::RIGHT) {
                output.remove(JoypadOutput::RIGHT_OR_A);
            }
            if self.input.contains(JoypadInput::LEFT) {
                output.remove(JoypadOutput::LEFT_OR_B);
            }
        }
        output.bits
    }
}
