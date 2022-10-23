use std::ops::Div;

use chip8_base::{self, Display, Pixel, Keys};
use log::info;

pub struct Interpreter {
    memory: [u8; 4096],
    V: [u8; 16],
    nibble_holder: (u8, u8, u8, u8),
    I: u16,
    PC: u16,
    delay_timer: u8,
    sound_timer: u8,
    SP: u8,
    stack: [u16; 16],
    clock_speed: u64,
    keys: [bool; 16],

    Display: Display,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter {
            memory: [0; 4096],
            V: [0; 16],
            nibble_holder: (0, 0, 0, 0),
            I: 0,
            PC: 0, //0x200
            delay_timer: 0,
            sound_timer: 0,
            SP: 0,
            stack: [0; 16],
            clock_speed: 700,
            Display: [[Pixel::Black; 64]; 32],
        }
    }

    fn fetch(&mut self) -> (u8, u8, u8, u8) {

        let opcode = 
            (
                (self.memory[self.PC as usize]) >> 4, (self.memory[self.PC as usize]) & 0xF,
                (self.memory[self.PC as usize + 1]) >> 4, (self.memory[self.PC as usize + 1]) & 0xF
            );
        //Increment PC
        self.PC += 2;
        
        if self.PC >= 4096 {
            self.PC = 0;
        }
        
        self.nibble_holder = opcode.clone();
        info!("FETCH {:?}, PC = {:?}", opcode, self.PC);
        opcode
    }

    fn execute(&mut self, opcode: (u8, u8, u8, u8)) {
        match opcode {
            (0x0, 0x0, 0xE, 0x0) => {
                self.Display = [[Pixel::Black; 64]; 32];
            },
            (0x0, 0x0, 0xE, 0xE) => {
                self.PC = self.stack[self.SP as usize];
                self.SP -= 1;
            },
            (0x0, x, y, z) => {
                //self.PC = ((x as u16) << 4 + (y as u16)) << 4 + (z as u16);
            },
            (0x1, x, y, z) => {
                self.PC = Interpreter::get_addr(x, y, z);
            },
            (0x2, x, y, z) => {
                self.SP += 1;
                self.stack[self.SP as usize] = self.PC;
                self.PC = Interpreter::get_addr(x, y, z);
            },
            (0x3, x, y, z) => {
                if self.V[x as usize] == Interpreter::get_byte(y, z) {
                    self.PC += 2;
                }
            },
            (0x4, x, y, z) => {
                if self.V[x as usize] != Interpreter::get_byte(y, z) {
                    self.PC += 2;
                }
            },
            (0x5, x, y, 0x0) => {
                if self.V[x as usize] == self.V[y as usize] {
                    self.PC += 2;
                }
            },
            (0x6, x, y, z) => {
                self.V[x as usize] = Interpreter::get_byte(y, z);
            },
            (0x7, x, y, z) => {
                self.V[x as usize] += Interpreter::get_byte(y, z);
            },
            (0x8, x, y, 0x0) => {
                self.V[x as usize] = self.V[y as usize];
            }
            (0x8, x, y, 0x1) => {
                self.V[x as usize] |= self.V[y as usize];
            },
            (0x8, x, y, 0x2) => {
                self.V[x as usize] &= self.V[y as usize];
            },
            (0x8, x, y, 0x3) => {
                self.V[x as usize] ^= self.V[y as usize];
            },
            (0x8, x, y, 0x4) => {
                let res = self.V[x as usize] as u16 + self.V[y as usize] as u16;

                if(res > 264) {
                    self.V[0xF] = 1;
                    res >>= 1;
                } else {
                    self.V[0xF] = 0;
                }
            },
            (0x8, x, y, 0x5) => {
                if(self.V[x as usize] > self.V[y as usize]) {
                    self.V[0xF] = 1;
                } else {
                    self.V[0xF] = 0;
                }
                
                self.V[x as usize] = self.V[x as usize].wrapping_sub(self.V[y as usize]);
            },
            (0x8, x, _y, 0x6) => {
                self.V[0xF] = self.V[x as usize] & 0x1;
                self.V[x as usize] >>= 1;
            },
            (0x8, x, y, 0x7) => {
                if(self.V[y as usize] > self.V[x as usize]) {
                    self.V[0xF] = 1;
                } else {
                    self.V[0xF] = 0;
                }
                
                self.V[x as usize] = self.V[y as usize].wrapping_sub(self.V[x as usize]);
            },
            (0x8, x, _y, 0xE) => {
                self.V[0xF] = self.V[x as usize] >> 7;
                self.V[x as usize] <<= 1;
            },
            (0x9, x, y, 0x0) => {
                if self.V[x as usize] != self.V[y as usize] {
                    self.PC += 2;
                }
            },
            (0xA, x, y, z) => {
                self.I = Interpreter::get_addr(x, y, z);
            },
            (0xB, x, y, z) => {
                self.PC = Interpreter::get_addr(x, y, z) + self.V[0] as u16;
            },
            (0xC, x, y, z) => {
                self.V[x as usize] = Interpreter::get_byte(y, z) & rand::random::<u8>();
            },
            (0xD, x, y, z) => {
                let x = self.V[x as usize] as usize % 64;
                let y = self.V[y as usize] as usize % 32;
                self.V[0xF] = 0;

                let height = (z as f64).div(8.0).ceil() as usize;
                let mut memory_line: u8;

                self.V[0xF] = 0;

                for yline in 0..height {
                    memory_line = self.memory[self.I as usize + yline];
                    for xline in 0..8 {
                        if (memory_line & (0x80 >> xline)) != 0 {
                            if self.Display[y + yline][x + xline] == Pixel::White {
                                self.V[0xF] = 1;
                            }
                            self.Display[y + yline][x + xline] ^= Pixel::White;
                        }
                    }
                }
            },
            (0xE, x, 0x9, 0xE) => {
                if self.keys[self.memory[x as usize] as usize] {
                    self.PC += 2;
                }
            },
            (0xE, x, 0xA, 0x1) => {
                if !self.keys[self.memory[x as usize] as usize] {
                    self.PC += 2;
                }
            },
            (0xF, x, 0x0, 0x7) => {
                self.V[x as usize] = self.delay_timer;
            },
            (0xF, x, 0x0, 0xA) => {
                todo!()
            },
            (0xF, x, 0x1, 0x5) => {
                self.delay_timer = self.memory[x as usize];
            },
            (0xF, x, 0x1, 0x8) => {
                self.sound_timer = self.memory[x as usize];
            },
            (0xF, x, 0x1, 0xE) => {
                self.I += self.memory[x as usize] as u16;
            },
            (0xF, x, 0x2, 0x9) => {
                self.I = self.memory[x as usize] as u16 * 5;
            },
            (0xF, x, 0x3, 0x3) => {
                todo!()
                self.memory[self.I as usize] = self.memory[x as usize] / 100;
                self.memory[self.I as usize + 1] = (self.memory[x as usize] / 10) % 10;
                self.memory[self.I as usize + 2] = (self.memory[x as usize] % 100) % 10;
            },
            (0xF, x, 0x5, 0x5) => {
                for i in 0..=x {
                    self.memory[self.I as usize + i] = self.memory[i as usize];
                }
            },
            (0xF, x, 0x6, 0x5) => {
                for i in 0..=x {
                    self.memory[i as usize] = self.memory[self.I as usize + i];
                }
            },
            }
        }
    }

    fn get_addr(x: u8, y: u8, z: u8) -> u16 {
        ((x as u16) << 8) | ((y as u16) << 4) | (z as u16)
    }

    fn get_byte(x: u8, y: u8) -> u8 {
        ((x as u8) << 4) | (y as u8)
    }
}


impl chip8_base::Interpreter for Interpreter {
    fn step(&mut self, keys: &Keys) -> Option<Display> {
        None
    }

    fn speed(&self) -> std::time::Duration {
        std::time::Duration::from_micros(1_000_000 / self.clock_speed)
    }

    fn buzzer_active(&self) -> bool {
        false
    }
}