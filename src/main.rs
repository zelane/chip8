use std::fs::File;
use std::io::prelude::*;
use hex;

fn main() -> std::io::Result<()> {
    let mut file = File::open("../games/pong.ch8")?;
    
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    // let mut opt_codes = Vec::new();

    for _byte_pair in buffer.chunks(2) {
        println!("{}", hex::encode(_byte_pair));
    }

    Ok(())
}
