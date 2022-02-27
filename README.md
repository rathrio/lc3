Little Computer 3 (LC-3) Virtual Machine
========================================

An LC-3 virtual machine written in Rust.

Prerequisites
-------------

- [Rust >= 2021](https://www.rust-lang.org/tools/install)

Running
-------

Run an example program with the debug build:

```sh
cargo run ./programs/rogue/rogue.obj
```

Running with an optimized build:

```sh
cargo build --release
./target/release/lc3 ./programs/rogue/rogue.obj
```

Resources
---------

- https://justinmeiners.github.io/lc3-vm/
- https://justinmeiners.github.io/lc3-vm/supplies/lc3-isa.pdf

Todo
----

- [x] Read hello world binary and print origin address
- [x] Handle traps

- [x] Implement LEA
- [x] Model registers

- [x] Refactor a bit 
    - [x] Put OP and TRAP in one file
    - [x] Extract functions for their handling

- [x] Implement some more instructions
- [x] Find out why blank strings are outputted

- [x] Fix character reading (validate with character counter program)
    - Noice, all programs except 2048 run

- [ ] Inspection capabilities
    - [ ] Inspect instructions
    - [ ] Inspect VM state

- [ ] Get 2048 to run
