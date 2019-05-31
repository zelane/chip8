use std::fs::File;
use std::io::prelude::*;

pub struct Cpu {
    ram: [u8; 4096],
    program_counter: usize,
    registers: [u8; 16],
    i: u16,
    stack: Vec<u8>
}

impl Cpu {
    pub fn new() -> Self {
        let mut ram = [0u8; 4096];
        let mut program_counter = 0x200;
        let mut registers = [0u8; 16];
        let mut stack = Vec::<u8>::new();
        let mut i = 0;

        Cpu {
            ram: ram,
            program_counter: program_counter,
            registers: registers,
            i: i,
            stack: stack,
        }
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

        println!("At {:x}", self.program_counter as u8);
        println!("Exec {:x}", opcode);

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
            (0x06, _, _, _) => {self.op_6xnn(x, value_u8);} // Set register
            (0x0a, _, _, _) => {self.op_annn(x);} // Set I
            (0x0d, _, _, _) => {self.op_dxyn(x, y, n);} // Draw sprite
            (0x0f, _, 0x03, 0x03) => {self.op_fx33(x);} // Do magic
            _ => {
                println!("Opcode not handled");
            }
        }
        true
    }

    pub fn op_00ee(&mut self) -> bool {
        self.program_counter = self.stack.pop().unwrap() as usize;
        true
    }

    pub fn op_1nnn(&mut self, address: usize) -> bool {
        println!("Jumps to address {}.", address);
        self.program_counter = address;
        true
    }

    pub fn op_2nnn(&mut self, address: usize) -> bool {
        println!("Call subroutine at {}.", address);
        self.stack.push((self.program_counter + 2) as u8);
        self.program_counter = address;
        true
    }

    pub fn op_6xnn(&mut self, register: usize, value: u8) -> bool {
        println!("Sets V{} to {}.", register, value);
        self.registers[register] = value;
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
}

fn main() -> std::io::Result<()> {
    let mut cpu = Cpu::new();

    cpu.load_program("games/pong.ch8".to_string());

    for i in 1..15 {
        cpu.exec();
    }

    println!("{:?}", cpu.ram);

    Ok(())
}
