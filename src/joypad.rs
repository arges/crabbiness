use bitflags::bitflags;

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
    strobe: bool
}

impl Controller {
    pub fn new() -> Self {
        Controller{
            index: 0,
            status: ControllerButtons::new(),
            strobe: false
        }
    }
}
