mod chip8_opcodes;
use std::fs::File;
use std::io::Read;
use std::time::{Duration, Instant};
use std::{u16, usize};

const IBM_LOGO: &str = "IBM Logo.ch8";

const CORAX_TEST: &str = "3-corax+.ch8";

//1000 / 60 = ~16
const TIMER_INTERVAL: Duration = Duration::from_millis(16);

struct Chip8 {
    memory: [u8; 4096],
    program_counter: u16,
    stack: [u16; 16],
    stack_pointer: usize,
    v_registers: [u8; 16], //this is what you call Vx in instructions
    v_flag: u8,
    i_register: u16,
    last_update: Instant,
    delay_timer: u8,
    sound_timer: u8,
    rom_size: u16,
}

impl Chip8 {
    fn new() -> Self {
        Chip8 {
            memory: [0; 4096],
            program_counter: 0,
            stack: [0; 16],
            stack_pointer: 0,
            v_registers: [0; 16],
            v_flag: 0,
            i_register: 0,
            last_update: Instant::now(),
            delay_timer: 0,
            sound_timer: 0,
            rom_size: 0,
        }
    }

    fn read_chip_8_file_as_bytes(&mut self) {
        let mut f: File = File::open(CORAX_TEST).expect("cannot find file");
        let mut byte_buffer: Vec<u8> = Vec::new();
        f.read_to_end(&mut byte_buffer).unwrap();
        self.rom_size = byte_buffer.len() as u16;
        let start_address: usize = 512; //0x200
        let end_address: usize = start_address + byte_buffer.len();
        self.memory[start_address..end_address].copy_from_slice(&byte_buffer);
    }

    fn update_timer(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_update) >= TIMER_INTERVAL {
            if self.delay_timer > 0 {
                self.delay_timer -= 1;
            }

            if self.sound_timer > 0 {
                //TODO: if this condition is met - let off sound
                self.sound_timer -= 1;
            }
        }
    }

    fn execute_opcode(&mut self, opcode: u16) {
        let instruction: u8 = (opcode >> 12) as u8;
        match instruction {
            0x0 => match opcode {
                0x00E0 => {
                    println!("(0E0) This Clears the screen");
                }
                0x00EE => {
                    println!("(0EE) Returns from a Subroutine");
                    self.program_counter = self.stack[self.stack_pointer];
                    if (self.stack_pointer != 0) {
                        self.stack_pointer -= 1;
                    }
                }
                _ => {
                    //TODO: Add catch arm for 0x0NNN and have catch all return unknown opcode
                    println!("(0_) NNN = {:X}", opcode & 0x0FFF);
                }
            },
            0x1 => {
                println!("(1) Jump to: {:X}", opcode & 0x0FFF);
                self.program_counter = opcode & 0x0FFF;
            }
            0x2 => {
                println!("(2) Calls Subroutine: {:X}", opcode & 0x0FFF);

                if self.stack_pointer >= self.stack.len() {
                    println!("stack overflow");
                }
                self.stack_pointer += 1;
                self.stack[self.stack_pointer] = self.program_counter;
                self.program_counter = opcode & 0x0FFF;
            }
            0x3 => {
                let vx: u8 = ((opcode & 0x0F00) >> 8) as u8;
                let nn: u8 = (opcode & 0x00FF) as u8;

                if (self.v_registers[vx as usize] == nn) {
                    self.program_counter += 2;
                }
            }
            0x4 => {
                let vx: u8 = ((opcode & 0x0F00) >> 8) as u8;
                let nn: u8 = (opcode & 0x00FF) as u8;

                if (self.v_registers[vx as usize] != nn) {
                    self.program_counter += 2;
                }
            }
            0x6 => {
                let vx: u8 = ((opcode & 0x0F00) >> 8) as u8;
                let nn: u8 = (opcode & 0x00FF) as u8;
                self.v_registers[vx as usize] = nn;
            }
            0x7 => {
                let vx: u8 = ((opcode & 0x0F00) >> 8) as u8;
                let nn: u16 = (opcode & 0x00FF);
                self.v_registers[vx as usize] = (self.v_registers[vx as usize] as u16 + nn) as u8;
            }
            0x8 => {
                //how should I split these into functions, probably one per suffix number but maybe
                //just one for 8 prefix and that method has a match

                //8 x y n
                let x: u16 = (opcode & 0x0F00) >> 8;
                let y: u16 = (opcode & 0x00F0) >> 4;
                let n: u16 = opcode & 0x000F;
                match n {
                    0 => self.v_registers[x as usize] = self.v_registers[y as usize],
                    1 => {
                        self.v_registers[x as usize] =
                            self.v_registers[x as usize] | self.v_registers[y as usize]
                    }
                    2 => {
                        self.v_registers[x as usize] =
                            self.v_registers[x as usize] & self.v_registers[y as usize]
                    }
                    3 => {
                        self.v_registers[x as usize] =
                            self.v_registers[x as usize] ^ self.v_registers[y as usize]
                    }
                    4 => {
                        let vx_plus_vy: u16 = self.v_registers[x as usize] as u16
                            + self.v_registers[y as usize] as u16; //grim - we can use a built in
                                                                   //rust func for calculating overflow
                        let is_carry: bool = vx_plus_vy > 255; //8 bit max value
                        self.v_flag = if is_carry { 1 } else { 0 };
                        self.v_registers[x as usize] = (self.v_registers[x as usize] as u16
                            + self.v_registers[y as usize] as u16)
                            as u8;
                    }
                    5 => {
                        //TODO: READ UP NEGATIVE NUMVER/OVERFLOW/WRAPPING IN CHIP8 - WE ARE SETTING
                        //V FLAG BUT I THINK WE NEED TO HANDLE THE WRAPPING AS WELL
                        let is_vx_greater: bool =
                            self.v_registers[x as usize] > self.v_registers[y as usize];
                        self.v_flag = if is_vx_greater { 1 } else { 0 };
                        self.v_registers[x as usize] = (self.v_registers[x as usize] as i16
                            - self.v_registers[y as usize] as i16)
                            as u8;
                    }
                    6 => {}
                    7 => {}
                    E => {}
                }
                println!("(8) {:X} | Not impled {:X}", n, opcode);
            }
            0xA => {
                println!("(A) Sets I to the address {:X}", opcode & 0x0FFF);
            }
            0xB => {
                println!("(B) Jumps to the address {} plus V0", opcode & 0x0FFF);
            }
            0xD => {
                println!(
                    "(D) Draw sprint at VX: {:X} VY: {:X} Height {:X}",
                    (opcode >> 8) & 0x0F,
                    (opcode >> 4) & 0x0F,
                    opcode & 0x0F
                );
            }
            0xE => {
                println!("(E) Not impled");
            }
            0xF => match opcode {
                _ => println!("(F) not finished"),
            },
            _ => {
                println!("(_) Not added yet: {:X}", opcode);
            }
        }
    }

    fn emulate_timer(&self) {}
}

fn main() {
    let mut chip_8 = Chip8::new();
    chip_8.read_chip_8_file_as_bytes();
    if true {
        println!("{}", chip_8.rom_size);
        for opcode in (512..512 + chip_8.rom_size).step_by(2) {
            chip_8.update_timer();
            //println!("{:04X}", opcode);
            //https://stackoverflow.com/questions/11193918/combine-merge-two-bytes-into-one
            let decoded_opcode = (chip_8.memory[opcode as usize] as u16) << 8
                | (chip_8.memory[opcode as usize + 1] as u16); //not a huge fan of the as sizes
                                                               //here
            chip_8.execute_opcode(decoded_opcode);
            println!(
                "COUNTERS: pc - {} | sp - {}",
                chip_8.program_counter, chip_8.stack_pointer
            );
            //println!("Timer: {} \n Opcode:{:04X}", chip_8.delay_timer, opcode);
            std::thread::sleep(TIMER_INTERVAL);
        }
    }
}

/*        for i in (0..byte_buffer.len()).step_by(2) {
            let opcode = (byte_buffer[i] as u16) << 8 | (byte_buffer[i + 1] as u16);
            println!("Opcode: {:04X}", opcode);
            self.rom_buffer.push(opcode);
        }
        for i in 0..self.memory.len() {
            println!("test {} - {:04X}", i, self.memory[i]);
        }
    rom_buffer: Vec<u16>,
            rom_buffer: Vec::new(),
*/
