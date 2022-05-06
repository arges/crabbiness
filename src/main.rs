#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

extern crate core;

use std::fs;
use std::io::Read;

mod bus;
mod cpu;
mod ppu;
mod rom;

fn main() {
    // load rom from disk
    let mut file = fs::File::open("mario.nes").unwrap();
    let mut data: Vec<u8> = Vec::new();
    file.read_to_end(&mut data).unwrap();
    let r = rom::Rom::new_from_ines(&data);

    // setup bus, cpu
    let bus = bus::Bus::new(r);
    let mut cpu = cpu::Cpu::new(bus);

    cpu.reset();
    cpu.run();
}
