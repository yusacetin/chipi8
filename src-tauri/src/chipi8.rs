use std::{sync::mpsc::Receiver, vec::Vec};
use std::collections::BTreeMap;
use rand::Rng;
use tauri::{AppHandle, Emitter};
use crate::EmulatorCommand;

pub struct Chipi8 {
    mem: [u8; 4096],
    reg: [u8; 16],
    i: u16,
    dt: u8,
    st: u8,
    pc: u16,
    sp: u8,
    display: [[bool; 64]; 32],
    stack: Vec::<u16>,
    keys: BTreeMap<u8, bool>,
    paused: bool,
    speed: u32
}

impl Chipi8 {
    const DISPLAY_WIDTH: usize = 64;
    const DISPLAY_HEIGHT: usize = 32;
    const PROGRAM_MEM_BEGIN: usize = 0x200;
    const SPRITE_0_ADDR: usize = 0x00;
    const SPRITE_1_ADDR: usize = 0x08;
    const SPRITE_2_ADDR: usize = 0x10;
    const SPRITE_3_ADDR: usize = 0x18;
    const SPRITE_4_ADDR: usize = 0x20;
    const SPRITE_5_ADDR: usize = 0x28;
    const SPRITE_6_ADDR: usize = 0x30;
    const SPRITE_7_ADDR: usize = 0x38;
    const SPRITE_8_ADDR: usize = 0x40;
    const SPRITE_9_ADDR: usize = 0x48;
    const SPRITE_A_ADDR: usize = 0x50;
    const SPRITE_B_ADDR: usize = 0x58;
    const SPRITE_C_ADDR: usize = 0x60;
    const SPRITE_D_ADDR: usize = 0x68;
    const SPRITE_E_ADDR: usize = 0x70;
    const SPRITE_F_ADDR: usize = 0x78;
}

impl Chipi8 {
    pub fn new() -> Self {
        let mut chipi8 = Self {
            mem: [0; 4096],
            reg: [0; 16],
            i: 0,
            dt: 0,
            st: 0,
            pc: 0x200,
            sp: 0,
            display: [[false; 64]; 32],
            stack: Vec::<u16>::new(),
            keys: BTreeMap::new(),
            paused: false,
            speed: 1
        };

        // Initialize keys and sprites
        {
            chipi8.keys.insert(0x0, false);
            chipi8.keys.insert(0x1, false);
            chipi8.keys.insert(0x2, false);
            chipi8.keys.insert(0x3, false);
            chipi8.keys.insert(0x4, false);
            chipi8.keys.insert(0x5, false);
            chipi8.keys.insert(0x6, false);
            chipi8.keys.insert(0x7, false);
            chipi8.keys.insert(0x8, false);
            chipi8.keys.insert(0x9, false);
            chipi8.keys.insert(0xA, false);
            chipi8.keys.insert(0xB, false);
            chipi8.keys.insert(0xC, false);
            chipi8.keys.insert(0xD, false);
            chipi8.keys.insert(0xE, false);
            chipi8.keys.insert(0xF, false);
        }
        chipi8.insert_preloaded_sprites();
        return chipi8;
    }

    pub fn display(&self, app_handle: &AppHandle) {
        let mut flat_display = [0u8; 2048];
        for row in 0..Chipi8::DISPLAY_HEIGHT {
            for col in 0..Chipi8::DISPLAY_WIDTH {
                if self.display[row][col] {
                    flat_display[row * Chipi8::DISPLAY_WIDTH + col] = 1;
                }
            }
        }
        app_handle.emit("draw-display", &flat_display[..]).expect("Failed to emit event");
    }

    pub fn read_rom(&mut self, fpath: &str) -> Result<bool, String> {
        let bytes = std::fs::read(fpath).map_err(|e| format!("Failed to read file: {e}"))?;
        for (i, byte) in bytes.iter().enumerate() {
            self.mem[Chipi8::PROGRAM_MEM_BEGIN + i] = *byte;
        }
        Ok(true)
    }

    pub fn set_speed(&mut self, new_speed: u32) {
        self.speed = new_speed;
    }

    pub fn run(&mut self, app_handle: AppHandle, rx: Receiver<EmulatorCommand>) {
        let mut loop_counter = 0u32;
        while self.pc < 4096 {
            // Check inputs
            while let Ok(cmd) = rx.try_recv() {
                match cmd {
                    EmulatorCommand::KeyPress(key) => {
                        self.keys.insert(key as u8, true);
                    },
                    EmulatorCommand::KeyRelease(key) => {
                        self.keys.insert(key as u8, false);
                    },
                    EmulatorCommand::SetPaused(is_paused) => {
                        self.paused = is_paused;
                    },
                    EmulatorCommand::Terminate => {
                        println!("Terminating running thread");
                        return;
                    },
                    EmulatorCommand::SetSpeed(speed) => {
                        self.speed = speed as u32;
                    }
                }
            }

            // Run CPU at 600 Hz
            std::thread::sleep(std::time::Duration::from_micros(1667 / self.speed as u64));

            // Handle paused state
            if self.paused {
                continue;
            }

            loop_counter += 1;

            // Update display and decrement timers at 60 Hz
            if loop_counter >= 10 {
                self.display(&app_handle);
                if self.dt > 0 {
                    self.dt -= 1;
                }
                if self.st > 0 {
                    self.st -= 1;
                    if self.st == 0 {
                        app_handle.emit("play-sound", false).expect("Failed to emit sound stop event");
                    }
                }
                loop_counter = 0;
            }

            // Fetch
            let msb = self.mem[self.pc as usize];
            let lsb = self.mem[(self.pc + 1) as usize];
            let instr: u16 = ((msb as u16) << 8) | (lsb as u16);
            let x = ((instr & 0xF00) >> 8) as usize;
            let y = ((instr & 0xF0) >> 4) as usize;
            let kk = lsb;
            let nnn = instr & 0xFFF;
            self.pc += 2;

            //println!("Current instruction: 0x{instr:04X}");
            //println!("New PC: {}", self.pc);

            // Decode and execute

            let msn = (msb & 0xF0) >> 4; // Most Significant Nibble

            if instr == 0 {
                println!("End of execution");
                break;
            }

            if instr == 0x00EE {
                // Return from function
                self.pc = match self.stack.pop() {
                    Some(n) => n,
                    None => {
                        println!("Failed to pop from stack");
                        break;
                    }
                };
                self.sp -= 1;
                continue;
            }

            if instr == 0x00E0 {
                // CLS: clear display
                self.display = [[false; 64]; 32];
                continue;
            }

            if msn == 0x1 {
                // JP addr: jump to address nnn
                self.pc = instr & 0xFFF;
                continue;
            }

            if msn == 0x2 {
                // CALL addr: call function at nnn
                self.sp += 1;
                self.stack.push(self.pc);
                self.pc = nnn;
                continue;
            }

            if msn == 0x3 {
                // Skip next instr if Vx == kk
                if self.reg[x] == kk {
                    self.pc += 2;
                }
                continue;
            }

            if msn == 0x4 {
                // Skip next instruction if Vx != kk
                if self.reg[x] != kk {
                    self.pc += 2;
                }
                continue;
            }

            if msn == 0x5 {
                // Skip next instruction if Vx == Vy
                if self.reg[x] == self.reg[y] {
                    self.pc += 2;
                }
                continue;
            }

            if msn == 0x6 {
                // LD x, kk: Set Vx to kk
                self.reg[x] = lsb;
                continue;
            }

            if msn == 0x7 {
                // ADD x, kk: Set Vx to Vx + kk
                self.reg[x] = self.reg[x].wrapping_add(kk);
                continue;
            }

            if msn == 0x8 {
                match lsb & 0xF {
                    0x0 => {
                        // Set Vx = Vy
                        self.reg[x] = self.reg[y];
                        continue;
                    },

                    0x1 => {
                        // OR
                        self.reg[x] |= self.reg[y];
                        continue;
                    },

                    0x2 => {
                        // AND
                        self.reg[x] &= self.reg[y];
                        continue;
                    },

                    0x3 => {
                        // XOR
                        self.reg[x] ^= self.reg[y];
                        continue;
                    },

                    0x4 => {
                        // ADD
                        // VF is carry
                        let (res, overflow) = self.reg[x].overflowing_add(self.reg[y]);
                        self.reg[x] = res;

                        if overflow {
                            self.reg[0xF] = 1;
                        } else {
                            self.reg[0xF] = 0;
                        }
                        continue;
                    },

                    0x5 => {
                        // SUB
                        // VF is NOT borrow
                        let (res, borrow) = self.reg[x].overflowing_sub(self.reg[y]);
                        self.reg[x] = res;

                        if borrow {
                            self.reg[0xF] = 0;
                        } else {
                            self.reg[0xF] = 1;
                        }
                        continue;
                    },

                    0x6 => {
                        // SHR (right shift by 1)
                        // VF is set to 1 if the least significant bit of Vx is 1, otherwise to 0
                        self.reg[0xF] = 0;
                        if self.reg[x] & 0b1 == 1 {
                            self.reg[0xF] = 1;
                        }
                        self.reg[x] >>= 1;
                        continue;
                    },

                    0x7 => {
                        // SUBN: Set Vx = Vy - Vx
                        // VF = NOT borrow
                        let (res, borrow) = self.reg[y].overflowing_sub(self.reg[x]);
                        self.reg[x] = res;

                        if borrow {
                            self.reg[0xF] = 0;
                        } else {
                            self.reg[0xF] = 1;
                        }
                        continue;
                    },

                    0xE => {
                        // SHL (left shift by 1)
                        // VF is set to 1 if the most significant bit of Vx is 1, otherwise to 0
                        self.reg[0xF] = 0;
                        if (self.reg[x] & 0x80) != 0 {
                            self.reg[0xF] = 1;
                        }
                        self.reg[x] <<= 1;
                        continue;
                    },

                    _ => {
                        println!("Invalid instruction: 0x{instr:04X}");
                        break;
                    }
                }
            }

            if msn == 0x9 {
                // Skip next instruction if Vx != Vy
                if self.reg[x] != self.reg[y] {
                    self.pc += 2;
                }
                continue;
            }

            if msn == 0xA {
                // LD I, addr: Set I = nnn
                self.i = instr & 0xFFF;
                continue;
            }

            if msn == 0xB {
                // Jump to location nnn + V0.
                self.pc = nnn + (self.reg[0] as u16);
                continue;
            }

            if msn == 0xC {
                // Set Vx = random byte AND kk
                let random_byte: u8 = rand::thread_rng().r#gen();
                let final_val = random_byte & kk;
                self.reg[x] = final_val;
                continue;
            }

            if msn == 0xD {
                // DRW x, y, n: display n-byte sprite from address I at (Vx, Vy), set VF = collision
                let vx = self.reg[x] as usize;
                let vy = self.reg[y] as usize;
                let n = (instr & 0xF) as usize;
                self.reg[0xF] = 0;
                
                for sprite_offset in 0..n {
                    let total_memory_offset = (self.i as usize) + sprite_offset;
                    let mut sprite = self.mem[total_memory_offset];
                    let display_y = (vy + sprite_offset) % Chipi8::DISPLAY_HEIGHT;
                    for bit_offset in 0..8 {
                        let display_x = (vx + bit_offset) % Chipi8::DISPLAY_WIDTH;
                        let bit = (sprite & 0x80) != 0;
                        if bit && self.display[display_y][display_x] {
                            self.reg[0xF] = 1;
                        }
                        self.display[display_y][display_x] ^= bit;
                        sprite <<= 1;
                    }
                }
                continue;
            }

            if msn == 0xE {
                if lsb == 0x9E {
                    // Skip next instruction if key with the value of Vx is pressed
                    let target = self.reg[x];
                    let is_target_pressed = *match self.keys.get(&target) {
                        Some(n) => n,
                        None => {
                            println!("Error: invalid key");
                            break;
                        }
                    };

                    if is_target_pressed {
                        self.pc += 2;
                    }

                    continue;
                }

                if lsb == 0xA1 {
                    // Skip next instruction if key with the value of Vx is not pressed
                    let target = self.reg[x];
                    let is_target_pressed = *match self.keys.get(&target) {
                        Some(n) => n,
                        None => {
                            println!("Error: invalid key");
                            break;
                        }
                    };

                    if !is_target_pressed {
                        self.pc += 2;
                    }

                    continue;
                }

                else {
                    println!("Invalid instruction: 0x{instr:04X}");
                    break;
                }
            }

            if msn == 0xF {
                if lsb == 0x07 {
                    // Set Vx = delay timer value
                    self.reg[x] = self.dt;
                    continue;
                }

                if lsb == 0x0A {
                    // Wait for a key press, store the value of the key in Vx
                    
                    let mut pressed_key = 0;
                    let mut find_success = true;
                    let mut pressed_success = false;
                    let mut i = 0;
                    while i < 16 {
                        let is_key_pressed = *match self.keys.get(&i) {
                            Some(n) => n,
                            None => {
                                println!("Error: invalid key");
                                find_success = false;
                                break;
                            }
                        };

                        if is_key_pressed {
                            pressed_success = true;
                            pressed_key = i;
                            break;
                        }

                        i += 1;

                        if i == 0 {
                            std::thread::sleep(std::time::Duration::from_millis(10));
                        }
                    }

                    if !find_success || !pressed_success {
                        self.pc -= 2;
                        continue;
                    }

                    self.reg[x] = pressed_key;
                    continue;
                }

                if lsb == 0x15 {
                    // Set delay timer = Vx
                    self.dt = self.reg[x];
                    continue;
                }
                
                if lsb == 0x18 {
                    // Set sound timer = Vx
                    self.st = self.reg[x];

                    // Play sound if sound timer > 1
                    // This should be processed here rather than at the top level because doing so at the top level
                    // would send many redundant events to the frontend every cycle when sound is active
                    if self.st > 1 {
                        app_handle.emit("play-sound", true).expect("Failed to emit sound start event");
                    }
                    continue;
                }

                if lsb == 0x1E {
                    // Set I = I + Vx
                    self.i += self.reg[x] as u16;
                    continue;
                }

                if lsb == 0x29 {
                    // Set I = location of sprite for digit Vx
                    match self.reg[x] {
                        0 => {
                            self.i = Chipi8::SPRITE_0_ADDR as u16;
                        },
                        1=> {
                            self.i = Chipi8::SPRITE_1_ADDR as u16;
                        },
                        2 => {
                            self.i = Chipi8::SPRITE_2_ADDR as u16;
                        },
                        3 => {
                            self.i = Chipi8::SPRITE_3_ADDR as u16;
                        },
                        4 => {
                            self.i = Chipi8::SPRITE_4_ADDR as u16;
                        },
                        5 => {
                            self.i = Chipi8::SPRITE_5_ADDR as u16;
                        },
                        6 => {
                            self.i = Chipi8::SPRITE_6_ADDR as u16;
                        },
                        7 => {
                            self.i = Chipi8::SPRITE_7_ADDR as u16;
                        },
                        8 => {
                            self.i = Chipi8::SPRITE_8_ADDR as u16;
                        },
                        9 => {
                            self.i = Chipi8::SPRITE_9_ADDR as u16;
                        },
                        10 => {
                            self.i = Chipi8::SPRITE_A_ADDR as u16;
                        },
                        11 => {
                            self.i = Chipi8::SPRITE_B_ADDR as u16;
                        },
                        12 => {
                            self.i = Chipi8::SPRITE_C_ADDR as u16;
                        },
                        13 => {
                            self.i = Chipi8::SPRITE_D_ADDR as u16;
                        },
                        14 => {
                            self.i = Chipi8::SPRITE_E_ADDR as u16;
                        },
                        15 => {
                            self.i = Chipi8::SPRITE_F_ADDR as u16;
                        },
                        _ => {
                            println!("Invalid sprite selection");
                            break;
                        }
                    }
                    continue;
                }

                if lsb == 0x33 {
                    // Store BCD representation of Vx in memory locations I, I+1, and I+2
                    let dec = self.reg[x];

                    // Calculate values
                    let hundreds: u8 = dec / 100;
                    let tens: u8 = (dec % 100) / 10;
                    let ones: u8 = dec % 10;

                    // Store values
                    self.mem[self.i as usize] = hundreds;
                    self.mem[(self.i as usize) + 1] = tens;
                    self.mem[(self.i as usize) + 2] = ones;

                    continue;
                }

                if lsb == 0x55 {
                    // Store registers V0 through Vx in memory starting at location I
                    for reg_index in 0..=x { // TODO equal or not?
                        self.mem[(self.i + (reg_index as u16)) as usize] = self.reg[reg_index];
                    }
                    continue;
                }

                if lsb == 0x65 {
                    // Read registers V0 through Vx from memory starting at location I
                    for reg_index in 0..=x { // TODO equal or not?
                        self.reg[reg_index] = self.mem[(self.i + (reg_index as u16)) as usize];
                    }
                    continue;
                }

                else {
                    println!("Invalid instruction: 0x{instr:04X}");
                    break;
                }
            }

            else {
                println!("Invalid instruction: 0x{instr:04X}");
                break;
            }
        }
    }

    fn insert_preloaded_sprites(&mut self) {
        self.mem[Chipi8::SPRITE_0_ADDR] = 0xF0;
        self.mem[Chipi8::SPRITE_0_ADDR + 1] = 0x90;
        self.mem[Chipi8::SPRITE_0_ADDR + 2] = 0x90;
        self.mem[Chipi8::SPRITE_0_ADDR + 3] = 0x90;
        self.mem[Chipi8::SPRITE_0_ADDR + 4] = 0xF0;

        self.mem[Chipi8::SPRITE_1_ADDR] = 0x20;
        self.mem[Chipi8::SPRITE_1_ADDR + 1] = 0x60;
        self.mem[Chipi8::SPRITE_1_ADDR + 2] = 0x20;
        self.mem[Chipi8::SPRITE_1_ADDR + 3] = 0x20;
        self.mem[Chipi8::SPRITE_1_ADDR + 4] = 0x70;

        self.mem[Chipi8::SPRITE_2_ADDR] = 0xF0;
        self.mem[Chipi8::SPRITE_2_ADDR + 1] = 0x10;
        self.mem[Chipi8::SPRITE_2_ADDR + 2] = 0xF0;
        self.mem[Chipi8::SPRITE_2_ADDR + 3] = 0x80;
        self.mem[Chipi8::SPRITE_2_ADDR + 4] = 0xF0;

        self.mem[Chipi8::SPRITE_3_ADDR] = 0xF0;
        self.mem[Chipi8::SPRITE_3_ADDR + 1] = 0x10;
        self.mem[Chipi8::SPRITE_3_ADDR + 2] = 0xF0;
        self.mem[Chipi8::SPRITE_3_ADDR + 3] = 0x10;
        self.mem[Chipi8::SPRITE_3_ADDR + 4] = 0xF0;

        self.mem[Chipi8::SPRITE_4_ADDR] = 0x90;
        self.mem[Chipi8::SPRITE_4_ADDR + 1] = 0x90;
        self.mem[Chipi8::SPRITE_4_ADDR + 2] = 0xF0;
        self.mem[Chipi8::SPRITE_4_ADDR + 3] = 0x10;
        self.mem[Chipi8::SPRITE_4_ADDR + 4] = 0x10;

        self.mem[Chipi8::SPRITE_5_ADDR] = 0xF0;
        self.mem[Chipi8::SPRITE_5_ADDR + 1] = 0x80;
        self.mem[Chipi8::SPRITE_5_ADDR + 2] = 0xF0;
        self.mem[Chipi8::SPRITE_5_ADDR + 3] = 0x10;
        self.mem[Chipi8::SPRITE_5_ADDR + 4] = 0xF0;

        self.mem[Chipi8::SPRITE_6_ADDR] = 0xF0;
        self.mem[Chipi8::SPRITE_6_ADDR + 1] = 0x80;
        self.mem[Chipi8::SPRITE_6_ADDR + 2] = 0xF0;
        self.mem[Chipi8::SPRITE_6_ADDR + 3] = 0x90;
        self.mem[Chipi8::SPRITE_6_ADDR + 4] = 0xF0;

        self.mem[Chipi8::SPRITE_7_ADDR] = 0xF0;
        self.mem[Chipi8::SPRITE_7_ADDR + 1] = 0x10;
        self.mem[Chipi8::SPRITE_7_ADDR + 2] = 0x20;
        self.mem[Chipi8::SPRITE_7_ADDR + 3] = 0x40;
        self.mem[Chipi8::SPRITE_7_ADDR + 4] = 0x40;

        self.mem[Chipi8::SPRITE_8_ADDR] = 0xF0;
        self.mem[Chipi8::SPRITE_8_ADDR + 1] = 0x90;
        self.mem[Chipi8::SPRITE_8_ADDR + 2] = 0xF0;
        self.mem[Chipi8::SPRITE_8_ADDR + 3] = 0x90;
        self.mem[Chipi8::SPRITE_8_ADDR + 4] = 0xF0;

        self.mem[Chipi8::SPRITE_9_ADDR] = 0xF0;
        self.mem[Chipi8::SPRITE_9_ADDR + 1] = 0x90;
        self.mem[Chipi8::SPRITE_9_ADDR + 2] = 0xF0;
        self.mem[Chipi8::SPRITE_9_ADDR + 3] = 0x10;
        self.mem[Chipi8::SPRITE_9_ADDR + 4] = 0xF0;

        self.mem[Chipi8::SPRITE_A_ADDR] = 0xF0;
        self.mem[Chipi8::SPRITE_A_ADDR + 1] = 0x90;
        self.mem[Chipi8::SPRITE_A_ADDR + 2] = 0xF0;
        self.mem[Chipi8::SPRITE_A_ADDR + 3] = 0x90;
        self.mem[Chipi8::SPRITE_A_ADDR + 4] = 0x90;

        self.mem[Chipi8::SPRITE_B_ADDR] = 0xE0;
        self.mem[Chipi8::SPRITE_B_ADDR + 1] = 0x90;
        self.mem[Chipi8::SPRITE_B_ADDR + 2] = 0xE0;
        self.mem[Chipi8::SPRITE_B_ADDR + 3] = 0x90;
        self.mem[Chipi8::SPRITE_B_ADDR + 4] = 0xE0;

        self.mem[Chipi8::SPRITE_C_ADDR] = 0xF0;
        self.mem[Chipi8::SPRITE_C_ADDR + 1] = 0x80;
        self.mem[Chipi8::SPRITE_C_ADDR + 2] = 0x80;
        self.mem[Chipi8::SPRITE_C_ADDR + 3] = 0x80;
        self.mem[Chipi8::SPRITE_C_ADDR + 4] = 0xF0;

        self.mem[Chipi8::SPRITE_D_ADDR] = 0xE0;
        self.mem[Chipi8::SPRITE_D_ADDR + 1] = 0x90;
        self.mem[Chipi8::SPRITE_D_ADDR + 2] = 0x90;
        self.mem[Chipi8::SPRITE_D_ADDR + 3] = 0x90;
        self.mem[Chipi8::SPRITE_D_ADDR + 4] = 0xE0;

        self.mem[Chipi8::SPRITE_E_ADDR] = 0xF0;
        self.mem[Chipi8::SPRITE_E_ADDR + 1] = 0x80;
        self.mem[Chipi8::SPRITE_E_ADDR + 2] = 0xF0;
        self.mem[Chipi8::SPRITE_E_ADDR + 3] = 0x80;
        self.mem[Chipi8::SPRITE_E_ADDR + 4] = 0xF0;

        self.mem[Chipi8::SPRITE_F_ADDR] = 0xF0;
        self.mem[Chipi8::SPRITE_F_ADDR + 1] = 0x80;
        self.mem[Chipi8::SPRITE_F_ADDR + 2] = 0xF0;
        self.mem[Chipi8::SPRITE_F_ADDR + 3] = 0x80;
        self.mem[Chipi8::SPRITE_F_ADDR + 4] = 0x80;
    }
}