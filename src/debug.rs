use crate::{instructions::Program, simulator::execute_statement};
use crossterm::{
    cursor,
    style::{PrintStyledContent, Stylize},
    terminal::{Clear, ClearType},
    ExecutableCommand,
};
use indexmap::IndexMap;
use std::{io::stdout, thread, time::Duration};

pub fn run_with_debug(program: &Program, registers: &mut IndexMap<String, usize>, pc: &mut usize) {
    let mut stdout = stdout();

    let mut count = 1;

    // Run the program
    loop {
        // Clear the screen and reset cursor position
        stdout.execute(Clear(ClearType::All)).unwrap();
        stdout.execute(cursor::MoveTo(0, 0)).unwrap();

        // Print program state
        println!("URM Debugger (step {})", count);

        stdout
            .execute(PrintStyledContent("============\n\n".to_string().grey()))
            .unwrap();

        println!("Registers:");
        for (reg, val) in &mut *registers {
            let styled = format!("{} = {}", reg, val).blue();
            stdout.execute(PrintStyledContent(styled)).unwrap();
            println!();
        }

        println!();
        println!("Instruction:");
        stdout
            .execute(PrintStyledContent(
                program.statements[*pc - 1].to_string(*pc - 1).blue(),
            ))
            .unwrap();
        println!();

        thread::sleep(Duration::from_millis(800));

        execute_statement(&program.statements[*pc - 1], registers, pc);

        count += 1;

        // Check if the program has terminated
        if *pc > program.statements.len() {
            break;
        }
    }

    println!()
}
