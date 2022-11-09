use interpreter::Interpreter;


mod interpreter;

fn main() {
    let mut interpreter  = Interpreter::new();

    let rom = std::fs::read("./roms/IBM Logo.ch8").unwrap();
    println!("READ ROM");
    println!("ROM SIZE: {}", rom.len());
    interpreter.load(rom);
    println!("LOADED ROM");

    println!("STARTING CHIP-8");
    chip8_base::run(interpreter)
}
