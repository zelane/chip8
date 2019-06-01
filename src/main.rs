use std::fs::File;
use std::io::prelude::*;
use hex;

pub struct Cpu {
    ram: [u8; 4096],
    program_counter: usize,
    registers: [u8; 16],
    i: u16,
    stack: Vec<u8>,

    delay_timer: u8,
    sound_timer: u8,
}

impl Cpu {
    pub fn new() -> Self {
        let mut ram = [0u8; 4096];
        let mut program_counter = 0x200;
        let mut registers = [0u8; 16];
        let mut stack = Vec::<u8>::new();
        let mut i = 0;

        let mut delay_timer = 0;
        let mut sound_timer = 0;

        Cpu::load_fonts(&mut ram);

        Cpu {
            ram: ram,
            program_counter: program_counter,
            registers: registers,
            i: i,
            stack: stack,
            delay_timer: delay_timer,
            sound_timer: sound_timer,
        }
    }

    fn load_fonts(ram: &mut [u8]) {
        let mut mem_index = 0;
        for _char in 0..15 {
            ram[mem_index+0] = 0xf0;
            ram[mem_index+1] = 0x90;
            ram[mem_index+2] = 0x90;
            ram[mem_index+3] = 0x90;
            ram[mem_index+4] = 0xf0;
            mem_index += 5;
        }
    }

    fn font_address(&mut self, font: u8) -> u16 {
        (font * 5) as u16
    }

    pub fn load_program(&mut self, file_path: String) -> bool {
        let mut file = File::open(file_path).expect("file not found");
        
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer);

        for (index, &_byte) in buffer.iter().enumerate() {
            self.ram[0x200 + index] = _byte;
        }
        
        true
    }

    pub fn exec(&mut self) -> bool {
        let opcode = (self.ram[self.program_counter] as u16) << 8 | (self.ram[self.program_counter + 1] as u16);

        println!("{:x}", opcode);

        let nibbles = (
            (opcode & 0xF000) >> 12 as u8,
            (opcode & 0x0F00) >> 8 as u8,
            (opcode & 0x00F0) >> 4 as u8,
            (opcode & 0x000F) as u8,
        );
        let address = (opcode & 0x0FFF) as usize;
        let value_u8 = (opcode & 0x00FF) as u8;
        let x = nibbles.1 as usize;
        let y = nibbles.2 as usize;
        let n = nibbles.3 as usize;

        match nibbles {
            (0x00, 0x00, 0x0e, 0x0e) => {self.op_00ee();} // Return from subroutine
            (0x01, _, _, _) => {self.op_1nnn(address);} // Jump to address
            (0x02, _, _, _) => {self.op_2nnn(address);} // Call subroutine
            (0x03, _, _, _) => {self.op_3xnn(x, value_u8);} // Skips the next instruction if VX equals NN.
            (0x06, _, _, _) => {self.op_6xnn(x, value_u8);} // Set register
            (0x07, _, _, _) => {self.op_7xnn(x, value_u8);} // Add value to register value
            (0x09, _, _, _) => {self.op_9xnn(x, y);} // Skip if VX != VY

            (0x0a, _, _, _) => {self.op_annn(x);} // Set I
            (0x0d, _, _, _) => {self.op_dxyn(x, y, n);} // Draw sprite
            (0x0f, _, 0x00, 0x07) => {self.op_fx07(x);} // Set VX to delay timer value
            (0x0f, _, 0x01, 0x05) => {self.op_fx15(x);} // Set delay timer
            (0x0f, _, 0x01, 0x08) => {self.op_fx18(x);} // Set sound timer
            (0x0f, _, 0x03, 0x03) => {self.op_fx33(x);} // Do magic
            (0x0f, _, 0x06, 0x05) => {self.op_fx65(x);} // Load from register
            (0x0f, _, 0x02, 0x09) => {self.op_fx29(x);} // Set I to loc of font in VX 
            _ => {
                println!("Opcode not handled");
            }
        }
        true
    }

    pub fn op_00ee(&mut self) -> bool {
        let return_address =  self.stack.pop().unwrap() as usize;
        println!("Return from subroutine to address {}.", return_address);

        // TODO: Move me to counter struct
        self.program_counter = 0x200 + return_address;
        self.program_counter += 2;

        true
    }

    pub fn op_1nnn(&mut self, address: usize) -> bool {
        println!("Jumps to address {}.", address);
        self.program_counter = address;
        true
    }

    pub fn op_2nnn(&mut self, address: usize) -> bool {
        println!("Call subroutine at {}.", address);
        self.stack.push((self.program_counter) as u8);
        self.program_counter = address;
        true
    }

    pub fn op_3xnn(&mut self, register: usize, value: u8) -> bool{
        println!("Skips the next instruction if V{} equals {}.", register, value);
        println!("{} == {}?", self.registers[register], value);
        if self.registers[register] == value {
            self.program_counter += 4;
        }
        else {
            self.program_counter += 2;
        }
        true
    }

    pub fn op_6xnn(&mut self, register: usize, value: u8) -> bool {
        println!("Sets V{} to {}.", register, value);
        self.registers[register] = value;
        self.program_counter += 2;
        true
    }

    pub fn op_7xnn(&mut self, register: usize, value: u8) -> bool {
        println!("Adds {} to V{}.", value, register);
        self.registers[register] += value;
        self.program_counter += 2;
        true
    }

    pub fn op_annn(&mut self, address: usize) -> bool {
        println!("Sets I to {}.", address as u16);
        self.i = address as u16;
        self.program_counter += 2;
        true
    }

    pub fn op_dxyn(&mut self, x: usize, y: usize, n: usize) -> bool {
        println!("Draws a sprite 8x{} at x{} y{}.", n, x, y);
        //  Each row of 8 pixels is read as bit-coded starting from memory location I; I value doesn’t change after the execution of this instruction. As described above, VF is set to 1 if any screen pixels are flipped from set to unset when the sprite is drawn, and to 0 if that doesn’t happen
        self.program_counter += 2;
        true
    }

    pub fn op_fx07(&mut self, register: usize) -> bool {
        println!("Sets V{} to the value of the delay timer.", register);
        self.registers[register] = self.delay_timer;
        self.program_counter += 2;
        true
    }

    // pub fn op_fx0A(&mut self, delay: usize) -> bool {
    //     println!("A key press is awaited, and then stored in V{}. (Blocking Operation. All instruction halted until next key event).", x);
    //     self.program_counter += 2;
    //     true
    // }

    pub fn op_fx15(&mut self, register: usize) -> bool {
        println!("Sets the delay timer to V{}.", register);
        self.delay_timer = self.registers[register];
        self.program_counter += 2;
        true
    }

    pub fn op_fx18(&mut self, register: usize) -> bool {
        println!("Sets the sound timer to V{}.", register);
        self.sound_timer = self.registers[register];
        self.program_counter += 2;
        true
    }

    pub fn op_fx33(&mut self, x: usize) -> bool {
        println!("Take the decimal representation of V{}, place the hundreds digit in memory at location in I, the tens digit at location I+1, and the ones digit at location I+2", x);
        let register_val = self.registers[x];

        self.ram[self.i as usize] = register_val / 100;
        self.ram[(self.i + 1) as usize] = (register_val % 100) / 10;
        self.ram[(self.i + 2) as usize] = register_val % 10;
        self.program_counter += 2;
        true
    }

    pub fn op_fx65(&mut self, x: usize) -> bool {
        println!("Fills V0 to V{} (including VX) with values from memory starting at address I. The offset from I is increased by 1 for each value written, but I itself is left unmodified.", x);
        
        let memory_index = self.i;

        for register in 0..x {
            let to_copy = self.ram[memory_index as usize];
            self.registers[register] = to_copy;
            println!("Copying {:x} -> register V{}", to_copy, register)
        }

        self.program_counter += 2;
        
        true
    }

    pub fn op_fx29(&mut self, x: usize) -> bool {
        println!("Set I to address of the font who's value is in V{}", x);
        let font = self.registers[x];
        self.i = self.font_address(font);

        self.program_counter += 2;
        true
    }

    pub fn op_9xnn(&mut self, x: usize, y: usize) -> bool{
        println!("Skips the next instruction if V{} doesn't equal V{}. (Usually the next instruction is a jump to skip a code block)", x, y);
        if self.registers[x] != self.registers[y] {
            self.program_counter += 4;
        }
        else {
            self.program_counter += 2;
        }
        true
    }
}

fn main() -> std::io::Result<()> {
    let mut cpu = Cpu::new();

    // Visualise code execution?

    cpu.load_program("games/pong.ch8".to_string());

    for i in 1..30 {
        cpu.exec();
        if cpu.delay_timer > 0 {
            cpu.delay_timer -= 1;
        }
        // cpu.sound_timer -= 1;
    }
    
    for (index, register) in cpu.registers.iter().enumerate() {
        println!("{}: {:x}", index, register);
    }
    // println!("{:?}", cpu.ram);

    Ok(())
}
