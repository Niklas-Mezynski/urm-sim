# URM Simulator in Rust

A simulator for Unlimited Register Machine (URM) programs, implemented in Rust.

## Installation

Build the binary:

```sh
cargo build --release
```

## Usage

```sh
./target/release/urm-sim-rust [--debug] <FILE> <INPUTS...>
```

Arguments:

- `<FILE>` Path to the URM program file
- `<INPUTS...>` Input values for the program's registers

Options:

- `-d`, `--debug` Enable debug mode to see step-by-step execution
- `-h`, `--help` Print help
- `-V`, `--version` Print version

## URM Program Syntax

URM programs consists of a input declaration, a sequence of 6 available instructions, and an output declaration:

```
in(R1, R2) # Input registers declaration
R1++; # Increment register
R2--; # Decrement register
R3 = 0; # Set register to zero
if R1 == 0 goto 6; # Conditional jump if register equals zero
if R2 != 0 goto 2; # Conditional jump if register is not zero
goto 1; # Unconditional jump
out(R1) # Output register declaration
```

## Examples

Addition program (urm-programs/add.urm):

```urm
in(R1, R2)
if R2 == 0 goto 5;
R2--;
R1++;
goto 1;
out(R1)
```

Run the addition program:

```sh
urm-sim-rust urm-programs/add.urm 5 3
```

Debug mode:

```sh
urm-sim-rust urm-programs/add.urm 5 3 --debug
```

## License

MIT
