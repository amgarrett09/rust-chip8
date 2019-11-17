use ggez::graphics::Image;
use ggez::{Context, GameResult};
use rand;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;

#[cfg(debug_assertions)]
macro_rules! debug {
    ($x:expr, $y:expr) => { println!($x, $y) }
}

#[cfg(not(debug_assertions))]
macro_rules! debug {
    ($x:expr, $y:expr) => {}
}

const FONT: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, 0x20, 0x60, 0x20, 0x20, 0x70, 0xF0, 0x10, 0xF0, 0x80, 0xF0, 0xF0,
    0x10, 0xF0, 0x10, 0xF0, 0x90, 0x90, 0xF0, 0x10, 0x10, 0xF0, 0x80, 0xF0, 0x10, 0xF0, 0xF0, 0x80,
    0xF0, 0x90, 0xF0, 0xF0, 0x10, 0x20, 0x40, 0x40, 0xF0, 0x90, 0xF0, 0x90, 0xF0, 0xF0, 0x90, 0xF0,
    0x10, 0xF0, 0xF0, 0x90, 0xF0, 0x90, 0x90, 0xE0, 0x90, 0xE0, 0x90, 0xE0, 0xF0, 0x80, 0x80, 0x80,
    0xF0, 0xE0, 0x90, 0x90, 0x90, 0xE0, 0xF0, 0x80, 0xF0, 0x80, 0xF0, 0xF0, 0x80, 0xF0, 0x80, 0x80,
];

pub struct Chip8 {
    io: IOState,
    cpu: CpuState,
}

struct IOState {
    key_inputs: [u8; 16],
    display_buffer: [u8; 8192],
    memory: [u8; 4096],
    stack: Vec<usize>,
}

struct CpuState {
    registers: [usize; 16],
    pc: usize,
    index: usize,
    sound_timer: usize,
    delay_timer: usize,
    waiting: bool,
    clock_speed: usize,
}

impl Chip8 {
    pub fn new(clock_speed: usize) -> Chip8 {
        let mut io = IOState {
            key_inputs: [0; 16],
            display_buffer: [255; 8192],
            memory: [0; 4096],
            stack: Vec::new(),
        };

        let cpu = CpuState {
            registers: [0; 16],
            pc: 512,
            index: 0,
            sound_timer: 0,
            delay_timer: 0,
            waiting: false,
            clock_speed: clock_speed,
        };

        // Load font
        for (i, ch) in FONT.iter().enumerate() {
            io.memory[i] = *ch;
        }

        // Clear screen
        for (i, pixel) in io.display_buffer.iter_mut().enumerate() {
            if (i + 1) % 4 == 0 {
                *pixel = 0;
            }
        }

        Chip8 { io: io, cpu: cpu }
    }

    pub fn load_rom(&mut self, path_string: &str) -> io::Result<()> {
        let path = Path::new(path_string);

        let mut rom = File::open(path)?;
        let mut buffer = Vec::new();
        rom.read_to_end(&mut buffer)?;

        for (i, byte) in buffer.iter().enumerate() {
            self.io.memory[i + 512] = *byte;
        }

        Ok(())
    }

    pub fn cycle(&mut self) {
        let mut counter = 0;

        while counter <  self.cpu.clock_speed / 60 {
            if self.cpu.pc > 4094 {
                return;
            }

            let high_byte = self.io.memory[self.cpu.pc] as usize;
            let low_byte = self.io.memory[self.cpu.pc + 1] as usize;

            let op_code = (high_byte << 8) | low_byte;

            self.cpu.pc += 2;
            process_opcode(&mut self.io, &mut self.cpu, op_code);
            debug!("op code: {:x?}", op_code);
            debug!("registers: {:?}", self.cpu.registers);
            debug!("program counter: {}", self.cpu.pc);
            debug!("index register: {}", self.cpu.index);
            debug!("key inputs: {:?}", self.io.key_inputs);

            counter += 1;
        }

        if self.cpu.delay_timer > 0 {
            self.cpu.delay_timer -= 1;
        }

        if self.cpu.sound_timer > 0 {
            self.cpu.sound_timer -= 1;
        }

        // TODO: play sound if sound timer != 0
    }

    pub fn image_from_display(&self, ctx: &mut Context) -> GameResult<Image> {
        Image::from_rgba8(ctx, 64, 32, &self.io.display_buffer)
    }

    pub fn press_key(&mut self, key: usize) {
        self.io.key_inputs[key] = 1;
    }

    pub fn unpress_key(&mut self, key: usize) {
        self.io.key_inputs[key] = 0;
    }
}

fn process_opcode(io: &mut IOState, cpu: &mut CpuState, op_code: usize) {
    // Match for op codes that don't have any variables
    match op_code {
        // CLS
        0x00E0 => {
            for pixel in io.display_buffer.iter_mut().skip(3) {
                *pixel = 0;
            }
            return;
        }
        // RET
        0x00EE => {
            match io.stack.pop() {
                Some(addr) => {
                    cpu.pc = addr;
                }
                None => panic!("Expected address to be on stack!"),
            }
            return;
        }
        _ => {}
    };

    // If none matched, check for op codes with variables
    let vx = (op_code & 0x0F00) >> 8;
    let vy = (op_code & 0x00F0) >> 4;
    let byte = op_code & 0xFF;

    match op_code & 0xF000 {
        // JMP nnn
        0x1000 => {
            cpu.pc = op_code & 0x0FFF;
        }
        // CALL nnn
        0x2000 => {
            io.stack.push(cpu.pc);
            cpu.pc = op_code & 0x0FFF;
        }
        // SE Vx, byte
        0x3000 => {
            if cpu.registers[vx] == byte {
                cpu.pc += 2;
            }
        }
        // SNE Vx, byte
        0x4000 => {
            if cpu.registers[vx] != byte {
                cpu.pc += 2;
            }
        }
        // SE Vx, Vy
        0x5000 => {
            if cpu.registers[vx] == cpu.registers[vy] {
                cpu.pc += 2;
            }
        }
        // LD Vx, byte
        0x6000 => {
            cpu.registers[vx] = byte;
        }
        // ADD Vx, byte
        0x7000 => {
            let mut result = cpu.registers[vx] + byte;
            result = result & 0xFF;

            cpu.registers[vx] = result;
        }
        0x8000 => {
            match op_code & 0x000F {
                // LD Vx, Vy
                0 => {
                    cpu.registers[vx] = cpu.registers[vy];
                }
                // OR Vx, Vy
                1 => {
                    let result = cpu.registers[vx] | cpu.registers[vy];
                    cpu.registers[vx] = result;
                }
                // AND Vx, Vy
                2 => {
                    let result = cpu.registers[vx] & cpu.registers[vy];
                    cpu.registers[vx] = result;
                }
                // XOR Vx, Vy
                3 => {
                    let result = cpu.registers[vx] ^ cpu.registers[vy];
                    cpu.registers[vx] = result;
                }
                // ADD Vx, Vy
                4 => {
                    let mut result = cpu.registers[vx] + cpu.registers[vy];
                    let mut carry = 0;

                    if result > 255 {
                        result = result & 0xFF;
                        carry = 1;
                    }

                    cpu.registers[vx] = result;
                    cpu.registers[15] = carry;
                }
                // SUB Vx, Vy
                5 => {
                    if cpu.registers[vx] > cpu.registers[vy] {
                        cpu.registers[15] = 1;
                    } else {
                        cpu.registers[15] = 0;
                    }

                    let mut result = (cpu.registers[vx] as isize) - (cpu.registers[vy] as isize);
                    result = result & 0xFF;

                    cpu.registers[vx] = result as usize;
                }
                // SHR Vx
                6 => {
                    cpu.registers[15] = cpu.registers[vx] & 0x01;
                    cpu.registers[vx] /= 2;
                }
                // SUBN Vx, Vy
                7 => {
                    if cpu.registers[vy] > cpu.registers[vx] {
                        cpu.registers[15] = 1;
                    } else {
                        cpu.registers[15] = 0;
                    }

                    let mut result = (cpu.registers[vy] as isize) - (cpu.registers[vx] as isize);
                    result = result & 0xFF;

                    cpu.registers[vx] = result as usize;
                }
                // SHL Vx
                14 => {
                    if cpu.registers[vx] & 0x80 > 0 {
                        cpu.registers[15] = 1;
                    } else {
                        cpu.registers[15] = 0;
                    }

                    let mut result = cpu.registers[vx] * 2;
                    result = result & 0xFF;

                    cpu.registers[vx] = result;
                }
                _ => {}
            }
        }
        // SNE Vx, Vy
        0x9000 => {
            if cpu.registers[vx] != cpu.registers[vy] {
                cpu.pc += 2;
            }
        }
        // LD I, addr
        0xA000 => {
            let addr = op_code & 0x0FFF;
            cpu.index = addr;
        }
        // JP V0, addr
        0xB000 => {
            let v0 = cpu.registers[0];
            let addr = op_code & 0x0FFF;

            cpu.pc = v0 + addr;
        }
        // RND Vx, byte
        0xC000 => {
            let mut rnd: usize = rand::random();
            rnd = rnd & 0xFF;
            cpu.registers[vx] = rnd & byte;
        }
        // DRW Vx, Vy, nibble
        0xD000 => {
            let nibble = op_code & 0x0F;
            cpu.registers[15] = draw_sprite(
                &io.memory,
                &mut io.display_buffer,
                cpu.index,
                nibble,
                (cpu.registers[vx], cpu.registers[vy]),
            );
        }
        0xE000 => match op_code & 0xFF {
            // SKP Vx
            0x9E => {
                if io.key_inputs[cpu.registers[vx]] > 0 {
                    cpu.pc += 2;
                }
            }
            // SKNP Vx
            0xA1 => {
                if io.key_inputs[cpu.registers[vx]] == 0 {
                    cpu.pc += 2;
                }
            }
            _ => {}
        },
        0xF000 => match op_code & 0xFF {
            // LD Vx, DT
            0x07 => {
                cpu.registers[vx] = cpu.delay_timer;
            }
            // LD Vx, K
            0x0A => {
                match io.key_inputs.iter().position(|&x| x == 1) {
                    Some(key) => {
                        cpu.waiting = false;
                        cpu.registers[vx] = key;
                    }
                    None => {
                        cpu.waiting = true;
                    }
                }

                if cpu.waiting {
                    cpu.pc -= 2;
                }
            }
            // LD DT, Vx
            0x15 => {
                cpu.delay_timer = cpu.registers[vx];
            }
            // LD ST, Vx
            0x18 => {
                cpu.sound_timer = cpu.registers[vx];
            }
            0x1E => {
                cpu.index = cpu.index + cpu.registers[vx];
            }
            // LD F, Vx
            0x29 => {
                cpu.index = (5 * cpu.registers[vx]) & 0xFFF;
            }
            // LD B, Vx
            0x33 => {
                let ones = cpu.registers[vx] % 10;
                let tens = (cpu.registers[vx] / 10) % 10;
                let hundreds = (cpu.registers[vx] / 100) % 10;

                io.memory[cpu.index] = hundreds as u8;
                io.memory[cpu.index + 1] = tens as u8;
                io.memory[cpu.index + 2] = ones as u8;
            }
            // LD [I], Vx
            0x55 => {
                let register_slice = &cpu.registers[0..(vx + 1)];

                for (i, byte) in register_slice.iter().enumerate() {
                    io.memory[i + cpu.index] = *byte as u8;
                }
            }
            // LD Vx, [I]
            0x65 => {
                let memory_slice = &io.memory[cpu.index..(cpu.index + vx + 1)];

                for (i, byte) in memory_slice.iter().enumerate() {
                    cpu.registers[i] = *byte as usize;
                }
            }
            _ => {}
        },
        _ => {}
    }
}

/* Draw sprite at (x, y) using data at the sprite index. The bits of the sprite
 * are XORed onto the screen, and if any pixels get erased, the vf register is set to 1.
 *
 * Returns value to be stored in vf */
fn draw_sprite(
    memory: &[u8],
    display_buffer: &mut [u8],
    sprite_index: usize,
    sprite_size: usize,
    coords: (usize, usize),
) -> usize {
    let mut vf = 0;
    let sprite = &memory[sprite_index..(sprite_index + sprite_size)];

    let (start_x, start_y) = coords;
    let mut x = start_x;
    let mut y = start_y;
    let w = 64; // width of screen

    for byte in sprite.iter() {
        // bit 7
        let sprite_bit = if byte & 0x80 > 0 { 1 } else { 0 };
        let result = xor_bits(sprite_bit, display_buffer, (x, y), w);
        if result == 1 {
            vf = 1;
        }

        x += 1;

        // bit 6
        let sprite_bit = if byte & 0x40 > 0 { 1 } else { 0 };
        let result = xor_bits(sprite_bit, display_buffer, (x, y), w);
        if result == 1 {
            vf = 1;
        }

        x += 1;

        // bit 5
        let sprite_bit = if byte & 0x20 > 0 { 1 } else { 0 };
        let result = xor_bits(sprite_bit, display_buffer, (x, y), w);
        if result == 1 {
            vf = 1;
        }

        x += 1;

        // bit 4
        let sprite_bit = if byte & 0x10 > 0 { 1 } else { 0 };
        let result = xor_bits(sprite_bit, display_buffer, (x, y), w);
        if result == 1 {
            vf = 1;
        }

        x += 1;

        // bit 3
        let sprite_bit = if byte & 0x08 > 0 { 1 } else { 0 };
        let result = xor_bits(sprite_bit, display_buffer, (x, y), w);
        if result == 1 {
            vf = 1;
        }

        x += 1;

        // bit 2
        let sprite_bit = if byte & 0x04 > 0 { 1 } else { 0 };
        let result = xor_bits(sprite_bit, display_buffer, (x, y), w);
        if result == 1 {
            vf = 1;
        }

        x += 1;

        // bit 1
        let sprite_bit = if byte & 0x02 > 0 { 1 } else { 0 };
        let result = xor_bits(sprite_bit, display_buffer, (x, y), w);
        if result == 1 {
            vf = 1;
        }

        x += 1;

        // bit 0
        let sprite_bit = if byte & 0x01 > 0 { 1 } else { 0 };
        let result = xor_bits(sprite_bit, display_buffer, (x, y), w);
        if result == 1 {
            vf = 1;
        }

        x = start_x;
        y += 1;
    }

    vf
}

/* We turn pixels in the rgba display buffer on/off by setting the alpha value to 255 or 0.
 * (x, y) coordinates get translated to indexes in the display buffer by the formula:
 * 4x + 4wy + 3
 * where w is the width of the image */
fn xor_bits(
    sprite_bit: u8,
    display_buffer: &mut [u8],
    coords: (usize, usize),
    width: usize,
) -> usize {
    let mut vf = 0;
    let (mut x, mut y) = coords;
    let w = width;

    // Wrap if we're out of bounds
    if x >= w {
        x = x % w;
    }

    if y >= 32 {
        y = y % 32;
    }

    let display_bit = if display_buffer[4 * x + 4 * w * y + 3] > 0 {
        1
    } else {
        0
    };

    let result = sprite_bit ^ display_bit;

    if result < display_bit {
        vf = 1;
    }

    display_buffer[4 * x + 4 * w * y + 3] = if result == 1 { 255 } else { 0 };

    vf
}
