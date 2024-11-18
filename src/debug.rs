use crate::{instructions::Program, simulator::execute_statement};
use crossterm::{
    cursor,
    style::{self, Color, PrintStyledContent, Stylize},
    terminal::{self, Clear, ClearType},
    ExecutableCommand,
};
use indexmap::IndexMap;
use std::io::{stdout, Write};
use std::{collections::HashMap, thread, time::Duration};

pub fn run_with_debug(program: &Program, registers: &mut IndexMap<String, usize>, pc: &mut usize) {
    let mut stdout = stdout();

    // Run the program
    loop {
        // Clear the screen and reset cursor position
        stdout.execute(Clear(ClearType::All)).unwrap();
        stdout.execute(cursor::MoveTo(0, 0)).unwrap();

        // Print program state
        println!("URM Debugger");
        println!("============");
        println!();

        println!("Registers:");
        for (reg, val) in &mut *registers {
            let styled = format!("{} = {}", reg, val).blue();
            stdout.execute(PrintStyledContent(styled)).unwrap();
            println!();
        }

        println!(
            "Instruction:\n{}",
            &program.statements[*pc - 1].to_string(*pc - 1)
        );

        thread::sleep(Duration::from_millis(800));

        execute_statement(&program.statements[*pc - 1], registers, pc);

        // Check if the program has terminated
        if *pc > program.statements.len() {
            break;
        }
    }
}
