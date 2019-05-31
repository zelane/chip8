use std::fs::File;
use std::io::prelude::*;
use hex;

pub struct Cpu {
    ram: [u8; 4096],
    program_counter: usize
}

impl Cpu {
    pub fn new() -> Self {
        let mut ram = [0u8; 4096];
        let mut program_counter = 0x200;

        Cpu {
            ram: ram,
            program_counter: program_counter
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

    pub fn run_opt_code(&mut self) -> bool {
        // let opt_code = hex::encode(byte_pair);
        let byte_pair = (self.ram[self.program_counter] as u16) << 8 | (self.ram[self.program_counter + 1] as u16);
        let opcode = format!("{:x}", byte_pair);
        let code: Vec<char> = opcode.chars().collect();
        let tuple = (code[0], code[1], code[2], code[3]);

        match tuple {
            ('1', _, _, _) => {self.op_1nnn(tuple);}
            ('6', _, _, _) => {self.op_6xnn(tuple);}
            _ => {}
        }
        true
    }

    pub fn op_1nnn(&mut self, _code: (char, char, char, char)) -> bool {
        println!("Jumps to address NNN.");
        true
    }

    pub fn op_6xnn(&mut self, _code: (char, char, char, char)) -> bool {
        println!("Sets VX to NN.");
        true
    }
}

fn main() -> std::io::Result<()> {
    let mut cpu = Cpu::new();

    cpu.load_program("games/pong.ch8".to_string());
    cpu.run_opt_code();

    Ok(())
}
