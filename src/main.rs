mod op;
mod vm;

use std::env;
use termios::*;
use vm::VM;

fn run(path_to_program: &str) {
    let program = std::fs::read(path_to_program).unwrap();
    let mut vm = VM::new(program);
    vm.run();
}

fn main() {
    // https://stackoverflow.com/questions/26321592/how-can-i-read-one-character-from-stdin-without-having-to-hit-enter
    let stdin = 0;
    let termios = Termios::from_fd(stdin).unwrap();
    let mut new_termios = termios;
    new_termios.c_iflag &= IGNBRK | BRKINT | PARMRK | ISTRIP | INLCR | IGNCR | ICRNL | IXON;
    new_termios.c_lflag &= !(ICANON | ECHO);
    tcsetattr(stdin, TCSANOW, &new_termios).unwrap();

    let args: Vec<String> = env::args().collect();
    run(&args[1]);

    tcsetattr(stdin, TCSANOW, &termios).unwrap();
}
