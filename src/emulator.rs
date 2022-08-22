use std::{fs::{File, self}, io::Read, collections::HashMap};
use crate::input::*;
use rand::Rng;

use rgraphics::{textures::RenderTexture2D, Program, colors};

const WIDTH: u8 = 64;
const HEIGHT: u8 = 32;

pub struct Chip8 {
    // 0x0 to 0x1FF reserved&
    // 0x200 (program start)
    // 0xFFF end
    memory: [u8; 4096],

    cps: u16,

    // Stack is an array of 16 16-bit values
    stack: [u16; 16],

    // General purpose, 8-bit registers
    registers: [u8; 16],

    // I register (usually for mem addresses)
    i: u16,

    // Delay timer
    dt: u8,

    // Sound timer
    st: u8,

    // Program counter
    pc: u16,

    // Stack pointer
    sp: u8,

    display: RenderTexture2D,

    key_map: HashMap<u8, Key>,
    sprite_locations: HashMap<u8, u16>,
}

impl Chip8 {

    fn load_sprites(&mut self) {
        let sprites = vec![
            0xF0, 0x90, 0x90, 0x90, 0xF0,
            0x20, 0x60, 0x20, 0x20, 0x70,
            0xF0, 0x10, 0xF0, 0x80, 0xF0,
            0xF0, 0x10, 0xF0, 0x10, 0xF0,
            0x90, 0x90, 0xF0, 0x10, 0x10,
            0xF0, 0x80, 0xF0, 0x10, 0xF0,
            0xF0, 0x80, 0xF0, 0x90, 0xF0,
            0xF0, 0x10, 0x20, 0x40, 0x40,
            0xF0, 0x90, 0xF0, 0x90, 0xF0,
            0xF0, 0x90, 0xF0, 0x10, 0xF0,
            0xF0, 0x90, 0xF0, 0x90, 0x90,
            0xE0, 0x90, 0xE0, 0x90, 0xE0,
            0xF0, 0x80, 0x80, 0x80, 0xF0,
            0xE0, 0x90, 0x90, 0x90, 0xE0,
            0xF0, 0x80, 0xF0, 0x80, 0xF0,
            0xF0, 0x80, 0xF0, 0x80, 0x80,
        ];

        for i in 0..sprites.len() {
            self.memory[i] = sprites[i];
        }
    }


    pub fn new(program: &Program) -> Chip8 {
        let mut key_map: HashMap<u8, Key> = HashMap::new();

        key_map.insert(0, Key::Key0);
        key_map.insert(1, Key::Key1);
        key_map.insert(2, Key::Key2);
        key_map.insert(3, Key::Key3);
        key_map.insert(4, Key::Key4);
        key_map.insert(5, Key::Key5);
        key_map.insert(6, Key::Key6);
        key_map.insert(7, Key::Key7);
        key_map.insert(8, Key::Key8);
        key_map.insert(9, Key::Key9);
        key_map.insert(10, Key::A);
        key_map.insert(11, Key::B);
        key_map.insert(12, Key::C);
        key_map.insert(13, Key::D);
        key_map.insert(14, Key::E);
        key_map.insert(15, Key::F);

        key_map.insert(0, Key::Numpad0);
        key_map.insert(1, Key::Numpad1);
        key_map.insert(2, Key::Numpad2);
        key_map.insert(3, Key::Numpad3);
        key_map.insert(4, Key::Numpad4);
        key_map.insert(5, Key::Numpad5);
        key_map.insert(6, Key::Numpad6);
        key_map.insert(7, Key::Numpad7);
        key_map.insert(8, Key::Numpad8);
        key_map.insert(9, Key::Numpad9);


        let mut loc_map: HashMap<u8, u16> = HashMap::new();

        for i in 0..16 {
            loc_map.insert(i, 5 * i as u16);
        }

        let mut c8 = Chip8 {
            memory: [0; 4096],
            stack: [0; 16],
            registers: [0; 16],
            cps: 500,
            i: 0,
            dt: 0,
            st: 0,
            pc: 0x200,
            sp: 0,
            display: RenderTexture2D::new(&program.renderer, WIDTH as u32, HEIGHT as u32),
            key_map,
            sprite_locations: loc_map,
        };

        c8.load_sprites();
        c8
    }

    pub fn draw(&mut self, program: &mut Program) {
        self.display.apply(&program.renderer);
        program.draw_texture(-1.0, 1.0, 2.0, 2.0, &self.display);
    }

    pub fn load_rom_from_file(&mut self, path: &str) {
        let mut f = File::open(&path).expect(format!("No file {}", path).as_str()); 
        let metadata = fs::metadata(&path).expect("Unable to read file metadata.");
        let mut buf = vec![0; metadata.len() as usize];
        f.read(&mut buf).expect("Buffer overflow.");
        self.load_rom_data(buf);
        
    }

    pub fn load_rom_data(&mut self, data: Vec<u8>) {
        for i in 0..data.len() {
            self.memory[0x200 + i] = data[i];
        }
    }

    pub fn set_cycles_per_second(&mut self, cycles: u16) {
        self.cps = cycles;
    }

    fn clear_display(&mut self) {
        for height in 0..self.display.dimensions.1 {
            for width in 0..self.display.dimensions.0 {
                self.display.set_pixel(width, height, colors::BLACK);
            }
        }
    }

    fn draw_sprite(&mut self, sprite: &Vec<u8>, x: u8, y: u8) {
        let mut collision = false; 

        for height in 0..sprite.len() {
            for width in 0..8 {
                let num = (sprite[height] >> (7 - width)) & 0x1;
                if num == 0 {
                    continue;
                }

                let (locx, locy) = (
                    (u8::wrapping_add(x, width)) % WIDTH,
                    (u8::wrapping_add(y, height as u8)) % HEIGHT,
                );

                if self.display.get_pixel(locx as u32, locy as u32).approx_equal(&colors::WHITE) {
                    self.display.set_pixel(locx as u32, locy as u32, colors::BLACK);
                    collision = true;
                }

                else {
                    self.display.set_pixel(locx as u32, locy as u32, colors::WHITE);
                }
            }
        }

        self.registers[15] = if collision {1} else {0}
    }

    pub fn clock(&mut self, program: &mut Program) {
        let start = std::time::Instant::now();
        self.run_single(program);
        while (std::time::Instant::now() - start).as_secs_f64() < (1.0 / self.cps as f64) {
            
        }
        self.draw(program);
    }

    fn run_single(&mut self, program: &mut Program) {
        // nnn or addr: lowest 12 bits
        // n or nibble: lowest 4 bits
        // x: lower 4 bits of high byte
        // y: upper 4 bits of low byte
        // kk or byte: lowest 8 bits

        let instruction: u16 = ((self.memory[self.pc as usize] as u16) << 8) | (self.memory[(self.pc + 1) as usize] as u16);
        let nnn = instruction & 0xFFF;
        let n = instruction & 0xF;
        let x = (instruction & 0xF00) >> 8;
        let y = (instruction & 0xF0) >> 4;
        let kk = instruction & 0xFF;

        if self.dt > 0 {
            self.dt -= 1;
        } else {self.dt = 0;}

        //println!("{:#06x}", instruction);

        match (instruction & 0xF000) >> 12 {
            0 => {
                match instruction {
                    0x00E0 => self.clear_display(),
                    0x00EE => {
                        self.pc = self.stack[self.sp as usize];
                        self.sp -= 1;
                    },
                    _ => (),
                }
            },

            1 => {
                self.pc = nnn;
                return;
            },
            2 => {
                self.sp += 1;
                self.stack[self.sp as usize] = self.pc;
                self.pc = nnn;
                return;
            },
            3 => {
                if self.registers[x as usize] == kk as u8 {self.pc += 2;}
            },
            4 => {
                if self.registers[x as usize] != kk as u8 {self.pc += 2;}
            },
            5 => {
                if self.registers[x as usize] == self.registers[y as usize] {self.pc += 2;}
            },
            6 => {
                self.registers[x as usize] = kk as u8;
            },
            7 => {
                self.registers[x as usize] = u8::wrapping_add(self.registers[x as usize], kk as u8);
            },
            8 => {
                match n {
                    0 => {self.registers[x as usize] = self.registers[y as usize]},
                    1 => {self.registers[x as usize] |= self.registers[y as usize]},
                    2 => {self.registers[x as usize] &= self.registers[y as usize]},
                    3 => {self.registers[x as usize] ^= self.registers[y as usize]},
                    4 => {
                        if self.registers[x as usize] as u16 + self.registers[y as usize] as u16 > 255 {self.registers[15] = 1} else {self.registers[15] = 0};

                        self.registers[x as usize] = u8::wrapping_add(self.registers[x as usize], self.registers[y as usize])
                    },
                    5 => {
                        let (n1, n2) = (self.registers[x as usize], self.registers[y as usize]);
                        if n1 > n2 {self.registers[15] = 1} else {self.registers[15] = 0}
                        self.registers[x as usize] = u8::wrapping_sub(n1, n2);
                    },
                    6 => {
                        if self.registers[x as usize] & 0x1 == 1 {self.registers[15] = 1;} else {self.registers[15] = 0;}
                        self.registers[x as usize] = u8::wrapping_div(self.registers[x as usize], 2);
                    },
                    7 => {
                        let (n1, n2) = (self.registers[x as usize], self.registers[y as usize]);
                        if n2 > n1 {self.registers[15] = 1} else {self.registers[15] = 0};
                        self.registers[x as usize] = u8::wrapping_sub(n2, n1);
                    },
                    0xE => {
                        if self.registers[x as usize] >> 7 == 1 {self.registers[15] = 1} else {self.registers[15] = 0};
                        self.registers[x as usize] = u8::wrapping_mul(self.registers[x as usize], 2);
                    },
                    _ => panic!("Unknown instruction"),
                }
            },
            9 => {
                match n {
                    0 => {if self.registers[x as usize] != self.registers[y as usize] {self.pc += 2;}},
                    1 => {self.registers[15] = if self.registers[x as usize] > self.registers[y as usize] {1} else {0}},
                    2 => {self.registers[15] = if self.registers[x as usize] >= self.registers[y as usize] {1} else {0}},
                    3 => {self.registers[15] = if self.registers[x as usize] < self.registers[y as usize] {1} else {0}},
                    4 => {self.registers[15] = if self.registers[x as usize] <= self.registers[y as usize] {1} else {0}},
                    _ => panic!("Unknown instruction.")
                }
            },
            0xA => {self.i = nnn},
            0xB => {self.pc = self.registers[0] as u16 + nnn as u16; return;},
            0xC => {
                let mut rng = rand::thread_rng();
                let rnum = rng.gen_range(0..256) as u8;
                self.registers[x as usize] = kk as u8 & rnum;
            },
            0xD => {
                let mut sprite: Vec<u8> = vec![];

                for i in 0..n {
                    sprite.push(self.memory[(self.i + i) as usize]);
                }

                self.draw_sprite(&sprite, self.registers[x as usize], self.registers[y as usize]);
            },
            0xE => {
                match kk {
                    0x9E => {
                        if program.input_manager.is_key_down(self.key_map[&self.registers[x as usize]]) {self.pc += 2;}
                    },
                    0xA1 => {
                        if !program.input_manager.is_key_down(self.key_map[&self.registers[x as usize]]) {self.pc += 2;}
                    },
                    _ => panic!("Unknown instruction")
                }
            },
            0xF => {
                match kk {
                    0x07 => {self.registers[x as usize] = self.dt},
                    0x0A => {
                        match program.input_manager.get_keyboard_events() {
                            Some(key) => {
                                let mut key_val: u8 = 0;

                                for entry in &self.key_map {
                                    if *entry.1 as u32 == key as u32 {
                                        key_val = *entry.0;
                                        break;
                                    }
                                }

                                self.registers[x as usize] = key_val;
                            },
                            _ => {
                                return;
                            }
                        }
                    },
                    0x15 => {self.dt = self.registers[x as usize]},
                    0x18 => {self.st = self.registers[x as usize]},
                    0x1E => {self.i += self.registers[x as usize] as u16},
                    0x29 => {
                        self.i = self.sprite_locations[&self.registers[x as usize]];
                    },
                    0x33 => {
                        let vx = self.registers[x as usize];

                        let n1 = vx / 100;
                        let n2 = (vx % 100) / 10;
                        let n3 = vx % 10;

                        self.memory[self.i as usize] = n1;
                        self.memory[self.i as usize + 1] = n2;
                        self.memory[self.i as usize + 2] = n3;
                    },
                    0x55 => {
                        for i in 0..=x {
                            self.memory[(self.i + i) as usize] = self.registers[i as usize];
                        }
                    },
                    0x65 => {
                        for i in 0..=x {
                            self.registers[i as usize] = self.memory[(i + self.i) as usize];
                        }
                    },
                    _ => panic!("Unknown instruction{:#06x}", instruction)
                }
            }

            _ => {}
        }

        self.pc += 2;

    }
}