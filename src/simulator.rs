use std::collections::HashSet;

use indexmap::IndexMap;

use crate::debug::run_with_debug;
use crate::instructions::*;

pub fn simulate_urm(program: &Program, input: Vec<usize>, debug: bool) -> usize {
    // Run static analysis
    match run_static_analysis(program, &input) {
        Ok(_) => {}
        Err(e) => {
            // This should never happen, as the parser should catch these errors
            eprintln!("Static analysis failed: {}", e);
            std::process::exit(1);
        }
    }

    // Initialize registers
    let mut registers: IndexMap<String, usize> =
        program.input_registers.iter().cloned().zip(input).collect();

    // Initialize program counter
    let mut pc: usize = 1;

    // Run the program
    match debug {
        true => run_with_debug(program, &mut registers, &mut pc).unwrap(),
        false => run_without_debug(program, &mut registers, &mut pc),
    };

    // Output the result
    let output_register = &program.output_register;
    let output_value = registers.get(output_register).unwrap_or(&0);

    *output_value
}

pub fn run_without_debug(
    program: &Program,
    registers: &mut IndexMap<String, usize>,
    pc: &mut usize,
) {
    // Run the program
    loop {
        execute_statement(&program.statements[*pc - 1], registers, pc);

        // Check if the program has terminated
        if *pc > program.statements.len() {
            break;
        }
    }
}

pub fn execute_statement(
    statement: &Statement,
    registers: &mut IndexMap<String, usize>,
    pc: &mut usize,
) {
    // Execute the statement
    match statement {
        Statement::Increment { register } => {
            let value = registers.get(register).unwrap_or(&0) + 1;
            registers.insert(register.clone(), value);
            *pc += 1;
        }
        Statement::Decrement { register } => {
            let value = registers
                .get(register)
                .unwrap_or(&0)
                .checked_sub(1)
                .unwrap_or(0);
            registers.insert(register.clone(), value);
            *pc += 1;
        }
        Statement::ZeroAssignment { register } => {
            registers.insert(register.clone(), 0);
            *pc += 1;
        }
        Statement::ConditionalGoto {
            register,
            condition,
            target,
        } => {
            let value = registers.get(register).unwrap_or(&0);
            let target_pc = *target;
            let new_pc = match condition {
                Condition::Equal => {
                    if value == &0 {
                        target_pc
                    } else {
                        *pc + 1
                    }
                }
                Condition::NotEqual => {
                    if value != &0 {
                        target_pc
                    } else {
                        *pc + 1
                    }
                }
            };
            *pc = new_pc;
        }
        Statement::Goto { target } => {
            *pc = *target;
        }
    }
}

pub fn run_static_analysis(program: &Program, input: &Vec<usize>) -> Result<(), String> {
    // Check if input registers are unique
    if program.input_registers.len() != program.input_registers.iter().collect::<HashSet<_>>().len()
    {
        return Err("Input registers must be unique".to_string());
    }

    // Check if length of input registers matches the length of the input vector
    if program.input_registers.len() != input.len() {
        return Err(format!("Input vector length does not match input register length. Program expects {} inputs, but {} were provided", program.input_registers.len(), input.len()));
    }

    Ok(())
}
