use bitflags::bitflags;
use macroquad::prelude::*;
use std::fmt::{self, Debug, Display, Formatter};

bitflags! {
    pub struct ControllerButtons: u8 {
        const RIGHT = 0b1000_0000;
        const LEFT = 0b0100_0000;
        const DOWN = 0b0010_0000;
        const UP = 0b0001_0000;
        const START = 0b0000_1000;
        const SELECT = 0b0000_0100;
        const B_BUTTON = 0b0000_0010;
        const A_BUTTON = 0b0000_0001;
    }
}

pub struct Controller {
    index: u8,
    status: ControllerButtons,
    strobe: bool,
}

impl Controller {
    pub fn new() -> Self {
        Controller {
            index: 0,
            status: ControllerButtons::from_bits_truncate(0),
            strobe: false,
        }
    }

    /// read bit at index and return if it is set. on strobe being off
    /// increment to the next index
    pub fn read(&mut self) -> u8 {
        if self.index > 7 {
            return 1;
        }
        let mut ret = 0;
        if (self.status.bits() & (1 << self.index)) > 0 {
            ret = 1;
        }

        if !self.strobe && self.index <= 7 {
            self.index += 1;
        }
        ret
    }

    /// if 1 is written set the strobe and reset index, otherwise clear strobe
    pub fn write(&mut self, data: u8) {
        if data == 1 {
            self.strobe = true;
            self.index = 0;
        } else {
            self.strobe = false;
        }
    }

    /// read keys directly from macroquad and set the status bits
    pub fn read_keys(&mut self) {
        self.status
            .set(ControllerButtons::A_BUTTON, is_key_down(KeyCode::A));
        self.status
            .set(ControllerButtons::B_BUTTON, is_key_down(KeyCode::S));
        self.status
            .set(ControllerButtons::SELECT, is_key_down(KeyCode::LeftShift));
        self.status
            .set(ControllerButtons::START, is_key_down(KeyCode::Enter));
        self.status
            .set(ControllerButtons::UP, is_key_down(KeyCode::Up));
        self.status
            .set(ControllerButtons::DOWN, is_key_down(KeyCode::Down));
        self.status
            .set(ControllerButtons::LEFT, is_key_down(KeyCode::Left));
        self.status
            .set(ControllerButtons::RIGHT, is_key_down(KeyCode::Right));
    }
}

impl fmt::Display for Controller {
    /// pretty print what has been pushed
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut result = String::new();
        if self.status.contains(ControllerButtons::A_BUTTON) {
            result.push('A');
        } else {
            result.push(' ');
        }
        if self.status.contains(ControllerButtons::B_BUTTON) {
            result.push('B');
        } else {
            result.push(' ');
        }
        if self.status.contains(ControllerButtons::SELECT) {
            result.push('L');
        } else {
            result.push(' ');
        }
        if self.status.contains(ControllerButtons::START) {
            result.push('S');
        } else {
            result.push(' ');
        }
        if self.status.contains(ControllerButtons::UP) {
            result.push('^');
        } else {
            result.push(' ');
        }
        if self.status.contains(ControllerButtons::DOWN) {
            result.push('V');
        } else {
            result.push(' ');
        }
        if self.status.contains(ControllerButtons::LEFT) {
            result.push('<');
        } else {
            result.push(' ');
        }
        if self.status.contains(ControllerButtons::RIGHT) {
            result.push('>');
        } else {
            result.push(' ');
        }
        write!(f, "{}", result)
    }
}
