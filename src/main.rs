use interpreter::Interpreter;


mod interpreter;

fn main() {
    let mut interpreter  = Interpreter::new();

    chip8_base::run(interpreter)
}
