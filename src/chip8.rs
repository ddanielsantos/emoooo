static CHIP8_FONT_SET: &[u8; 80] = &[
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

struct Chip8 {
    current_opcode: u16,
    /// The Chip-8 language is capable of accessing up to 4KB (4,096 bytes) of RAM, from location 0x000 (0) to 0xFFF (4095). The first 512 bytes, from 0x000 to 0x1FF, are where the original interpreter was located, and should not be used by programs.
    memory: [u8; 4096],
    /// Chip-8 has 16 general purpose 8-bit registers, usually referred to as Vx, where x is a hexadecimal digit (0 through F).
    v: [u8; 16],
    /// There is also a 16-bit register called I. This register is generally used to store memory addresses, so only the lowest (rightmost) 12 bits are usually used.
    index_register: u16,
    /// Used to store the currently executing address.
    program_counter: u16,
    /// The original implementation of the Chip-8 language used a 64x32-pixel monochrome display.
    gfx: [u8; 64 * 32],
    /// The delay timer is active whenever the delay timer register (DT) is non-zero. This timer does nothing more than subtract 1 from the value of DT at a rate of 60Hz. When DT reaches 0, it deactivates.
    delay_timer: u8,
    /// The sound timer is active whenever the sound timer register (ST) is non-zero. This timer also decrements at a rate of 60Hz, however, as long as ST's value is greater than zero, the Chip-8 buzzer will sound. When ST reaches zero, the sound timer deactivates.
    sound_timer: u8,
    /// Used to point to the topmost level of the stack.
    stack_pointer: u16,
    stack: [u16; 16],
}

const LAST_12_BITS_MASK: u16 = 0x0FFF;
const LAST_8_BITS_MASK: u16 = 0x00FF;
const FIRST_4_BITS_MASK: u16 = 0xF000;
const SECOND_4_BITS_MASK: u16 = 0x0F00;
const THIRD_4_BITS_MASK: u16 = 0x00F0;
const FOURTH_4_BITS_MASK: u16 = 0x000F;
/// Most Chip-8 programs start at location 0x200 (512), but some begin at 0x600 (1536). Programs beginning at 0x600 are intended for the ETI 660 computer.
const CHIP8_PROGRAM_OFFSET: u16 = 0x200;
const ETI660_PROGRAM_OFFSET: u16 = 0x600;

enum ProgramKind {
    CHIP8,
    ETI660
}

const F: usize = 0xF;

impl Chip8 {
    pub fn initialize(&mut self) {
        self.program_counter = 0x200;
        self.current_opcode = 0;
        self.index_register = 0;
        self.stack_pointer = 0;

        // The data should be stored in the interpreter area of Chip-8 memory
        for (i, item) in CHIP8_FONT_SET.iter().enumerate().take(80) {
            self.memory[i] = *item;
        }
    }

    pub fn load_program(&mut self, program: &[u8], kind: ProgramKind) {
        match kind {
            ProgramKind::CHIP8 => self.load_chip8_program(program),
            ProgramKind::ETI660 => self.load_eti660_program(program),
        }
    }

    fn load_chip8_program(&mut self, program_buffer: &[u8]) {
        for (i, &byte) in program_buffer.iter().enumerate() {
            self.memory[(CHIP8_PROGRAM_OFFSET as usize) + i] = byte;
        }
    }

    fn load_eti660_program(&mut self, program_buffer: &[u8]) {
        for (i, &byte) in program_buffer.iter().enumerate() {
            self.memory[(ETI660_PROGRAM_OFFSET as usize) + i] = byte;
        }
    }

    fn fetch_opcode(&mut self) {
        self.current_opcode = u16::from_be_bytes([self.memory[self.program_counter as usize], self.memory[(self.program_counter + 1) as usize]])
    }

    fn decode_opcode(&mut self) {
        let first_1_n = self.current_opcode & FIRST_4_BITS_MASK;
        let x = (self.current_opcode & SECOND_4_BITS_MASK) >> 8;
        let y = (self.current_opcode & THIRD_4_BITS_MASK) >> 4;
        let fourth_1_n = self.current_opcode & FOURTH_4_BITS_MASK;
        let last_2_n: u8 = (self.current_opcode & LAST_8_BITS_MASK) as u8;
        let last_3_n = self.current_opcode & LAST_12_BITS_MASK;

        match first_1_n {
            0x0000 => match self.current_opcode {
                // CLS
                0x00E0 => self.clear_screen(),
                // RET
                0x00EE => {
                    self.program_counter = self.stack[self.stack_pointer as usize];
                    self.stack_pointer -= 1;
                },
                // 0NNN
                _ => {
                    println!("calling machine routine for {}", last_3_n);
                }
            },
            // JP
            0x1000 => {
                self.program_counter = last_3_n;
            },
            // CALL addr
            0x2000 => {
                self.stack_pointer += 1;
                self.stack[self.stack_pointer as usize] = self.program_counter;
                self.program_counter = last_3_n;
            },
            // SE Vx, byte
            0x3000 => {
                if self.v[x as usize] == last_2_n {
                    self.program_counter += 2;
                }
            }
            // SNE Vx, byte
            0x4000 => {
                if self.v[x as usize] != last_2_n {
                    self.program_counter += 2;
                }
            }
            // SE Vx, Vy
            0x5000 => {
                if self.v[x as usize] == self.v[y as usize] {
                    self.program_counter += 2;
                }
            }
            // LD Vx, byte
            0x6000 => {
                self.v[x as usize] = last_2_n;
            }
            // ADD Vx, byte
            0x7000 => {
                self.v[x as usize] += last_2_n;
            }
            0x8000 => {
                match fourth_1_n {
                    // LD Vx, Vy
                    0x0 => self.v[x as usize] = self.v[y as usize],
                    // OR Vx, Vy
                    0x1 => self.v[x as usize] |= self.v[y as usize],
                    // AND Vx, Vy
                    0x2 => self.v[x as usize] &= self.v[y as usize],
                    // XOR Vx, Vy
                    0x3 => self.v[x as usize] ^= self.v[y as usize],
                    // ADD Vx, Vy
                    0x4 => {
                        let (sum, overflowing) = self.v[x as usize].overflowing_add(self.v[y as usize]);
                        self.v[x as usize] = sum;
                        self.v[F] = if overflowing { 1 } else { 0 }
                    },
                    // SUB Vx, Vy
                    0x5 => {
                        let x_bigger = self.v[x as usize] > self.v[y as usize];
                        self.v[F] = if x_bigger { 1 } else { 0 }
                    },
                    // SHR Vx {, Vy}
                    0x6 => {
                        let x_lsb = self.v[x as usize] & 1;
                        self.v[F] = if x_lsb == 1 { 1 } else { 0 };
                        self.v[x as usize] >>= 1;
                    },
                    // SUBN Vx, Vy
                    0x7 => {
                        let y_bigger = self.v[y as usize] > self.v[x as usize];
                        self.v[F] = if y_bigger { 1 } else { 0 }
                    },
                    // SHL Vx {, Vy}
                    0xE => {
                        let x_msb = (self.v[x as usize] >> 7) & 1;
                        self.v[F] = if x_msb == 1 { 1 } else { 0 };
                        self.v[x as usize] <<= 1;
                    },
                    _ => todo!()
                }
            }
            // SNE Vx, Vy
            0x9000 => {
                if self.v[x as usize] != self.v[y as usize] {
                    self.program_counter += 2;
                }
            }
            // LD I, addr
            0xA000 => {
                self.index_register = last_3_n;
            }
            // JP V0, addr
            0xB000 => {
                self.program_counter = last_3_n + self.v[0] as u16;
            }
            _ => {
                eprint!("Invalid opcode 0x{:X}", self.current_opcode);
            }
        }
    }

    /// Chip-8 also has two special purpose 8-bit registers, for the delay and sound timers. When these registers are non-zero, they are automatically decremented at a rate of 60Hz.
    fn update_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                println!("BEEP!");
            }
            self.sound_timer -= 1;
        }
    }

    fn clear_screen(&mut self) {
        todo!()
    }
}
