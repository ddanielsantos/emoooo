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
    program_counter: u16,
    gfx: [u8; 64 * 32],
    delay_timer: u8,
    sound_timer: u8,
    stack_pointer: u16,
    stack: [u16; 16],
}

const LAST_12_BITS_MASK: u16 = 0x0FFF;
const FIRST_4_BITS_MASK: u16 = 0xF000;
/// Most Chip-8 programs start at location 0x200 (512), but some begin at 0x600 (1536). Programs beginning at 0x600 are intended for the ETI 660 computer.
const PROGRAM_OFFSET: u16 = 0x200;

impl Chip8 {
    pub fn initialize(&mut self) {
        self.program_counter = 0x200;
        self.current_opcode = 0;
        self.index_register = 0;
        self.stack_pointer = 0;

        // load font-set
        for (i, item) in CHIP8_FONT_SET.iter().enumerate().take(80) {
            self.memory[i] = *item;
        }
    }

    pub fn load_program(&mut self, program_buffer: &[u8]) {
        for (i, &byte) in program_buffer.iter().enumerate() {
            self.memory[(PROGRAM_OFFSET as usize) + i] = byte;
        }
    }

    fn fetch_opcode(&mut self) {
        self.current_opcode = u16::from_be_bytes([self.memory[self.program_counter as usize], self.memory[(self.program_counter + 1) as usize]])
    }

    fn decode_opcode(&mut self) {
        let first_four_bits = self.current_opcode & FIRST_4_BITS_MASK;
        match first_four_bits {
            0x0000 => match self.current_opcode {
                // CLS
                0x00E0 => self.clear_screen(),
                // RET
                0x00EE => todo!("RETURNS A SUBROUTINE"),
                // 0NNN
                _ => {
                    let addr = self.current_opcode & LAST_12_BITS_MASK;
                    println!("calling machine routine for {}", addr);
                }
            },
            // JP
            0x1000 => {
                let addr = self.current_opcode & LAST_12_BITS_MASK;
                self.program_counter = addr;
            },
            0x2000 => {

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
