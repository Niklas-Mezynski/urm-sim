use clap::Parser;

pub mod instructions;
pub mod parser;
pub mod simulator;

/// URM code parser and interpreter
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Filename or path to the URM program
    #[arg(index = 1)]
    file: String,

    /// Values for the input registers
    #[arg(index = 2)]
    inputs: Vec<usize>,
}

fn main() {
    let args = Args::parse();

    // Read the URM code from the file
    let urm_code = match std::fs::read_to_string(&args.file) {
        Ok(urm_code) => urm_code,
        Err(e) => {
            eprintln!("Failed to read input file: {}", e);
            std::process::exit(1);
        }
    };

    parse_and_execute(urm_code.as_str(), args.inputs);
}

fn parse_and_execute(urm_code: &str, input: Vec<usize>) {
    let program = match parser::parse_urm_code(urm_code) {
        Ok(program) => program,
        Err(e) => {
            eprintln!("Failed to parse: {}", e);
            std::process::exit(1);
        }
    };

    let program_result = simulator::simulate_urm(&program, input);

    println!("Program result: {}", program_result);
}
