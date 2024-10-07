use std::fs::File;
use std::io::Read;
use std::time::{Duration, Instant};

//1000 / 60 = ~16
const TIMER_INTERVAL: Duration = Duration::from_millis(16);

struct Chip8 {
    memory: [u8; 4096],
    stack: [u16; 16],
    stack_pointer: usize,
    last_update: Instant,
    delay_timer: u8,
    sound_timer: u8,
    rom_buffer: Vec<u16>,
}

impl Chip8 {
    fn new() -> Self {
        Chip8 {
            memory: [0; 4096],
            stack: [0; 16],
            stack_pointer: 0,
            last_update: Instant::now(),
            delay_timer: 0,
            sound_timer: 0,
            rom_buffer: Vec::new(),
        }
    }

    fn read_chip_8_file_as_bytes(&mut self) {
        let mut f: File = File::open("IBM Logo.ch8").expect("cannot find file");
        let mut byte_buffer: Vec<u8> = Vec::new();
        f.read_to_end(&mut byte_buffer).unwrap();
        for i in (0..byte_buffer.len()).step_by(2) {
            let opcode = (byte_buffer[i] as u16) << 8 | (byte_buffer[i + 1] as u16);
            println!("Opcode: {:04X}", opcode);
            self.rom_buffer.push(opcode);
        }
    }

    fn update_timer(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_update) >= TIMER_INTERVAL {
            if self.delay_timer > 0 {
                self.delay_timer -= 1;
            }

            if self.sound_timer > 0 {
                self.sound_timer -= 1;
            }
        }
    }

    fn execute_opcode(&self, opcode: &u16) {
        let instruction: u8 = (opcode >> 12) as u8;
        match instruction {
            0x0 => match opcode {
                0x00E0 => {
                    println!("This Clears the screen");
                }
                0x00EE => {
                    println!("Returns from a Subroutine");
                }
                _ => {
                    //TODO: Add catch arm for 0x0NNN and have catch all return unknown opcode
                    println!("NNN = {:X}", opcode & 0x0FFF);
                }
            },
            0x1 => {
                println!("Jump to: {:X}", opcode & 0x0FFF);
            }
            0x2 => {
                println!("Calls Subroutine: {:X}", opcode & 0x0FFF);
            }
            0x6 => {
                println!("Sets V {:X} to {:X}", (opcode >> 8) & 0x0F, opcode & 0x00FF);
            }
            0xA => {
                println!("Sets I to the address {:X}", opcode & 0x0FFF);
            }
            0xB => {
                println!("Jumps to the address {} plus V0", opcode & 0x0FFF);
            }
            0xD => {
                println!(
                    "Draw sprint at VX: {:X} VY: {:X} Height {:X}",
                    (opcode >> 8) & 0x0F,
                    (opcode >> 4) & 0x0F,
                    opcode & 0x0F
                );
            }
            0xF => match opcode {
                _ => println!("F not finished"),
            },
            _ => {
                println!("Not added yet: {:X}", opcode);
            }
        }
    }

    fn emulate_timer(&self) {}
}

fn main() {
    let mut chip_8 = Chip8::new();
    chip_8.read_chip_8_file_as_bytes();
    if true {
        for opcode in &chip_8.rom_buffer.clone() {
            chip_8.update_timer();
            chip_8.execute_opcode(opcode);
            //           println!("Timer: {} \n Opcode:{:04X}", chip_8.delay_timer, opcode);
            std::thread::sleep(TIMER_INTERVAL);
        }
    }
}
