use crate::{instructions::Program, simulator::execute_statement};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute,
    style::{PrintStyledContent, Stylize},
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
    ExecutableCommand,
};
use indexmap::IndexMap;
use std::io::Write;
use std::{io::stdout, ops::ControlFlow, time::Duration};

const DEFAULT_TIMEOUT_MILLIS: u64 = 1500;

#[derive(Debug)]
pub enum DebugMode {
    Auto { timeout: u64 },
    Manual { step: bool },
}

pub fn run_with_debug(program: &Program, registers: &mut IndexMap<String, usize>, pc: &mut usize) {
    let mut count = 1;

    let mut last_execution = std::time::Instant::now();

    let mut debug_state = DebugMode::Manual { step: false };

    enable_raw_mode().unwrap();
    let mut stdout = stdout();
    stdout.execute(cursor::SavePosition).unwrap();

    loop {
        execute!(
            stdout,
            cursor::RestorePosition,
            Clear(ClearType::FromCursorDown),
        )
        .unwrap();

        print_debug_header(&count, &mut stdout);
        print_registers_state(registers, &mut stdout);
        print_current_instruction(program, *pc, &mut stdout);
        print_tooltip(&mut stdout, &debug_state);
        stdout.flush().unwrap();

        get_input(&mut debug_state);

        match debug_state {
            DebugMode::Auto { timeout } => {
                if last_execution.elapsed() < Duration::from_millis(timeout) {
                    continue;
                }
                last_execution = std::time::Instant::now();

                if let ControlFlow::Break(_) =
                    execute_next_statement(program, pc, registers, &mut count)
                {
                    break;
                }
            }
            DebugMode::Manual { step } => {
                if step {
                    debug_state = DebugMode::Manual { step: false };

                    if let ControlFlow::Break(_) =
                        execute_next_statement(program, pc, registers, &mut count)
                    {
                        break;
                    }
                }
            }
        }
    }

    disable_raw_mode().unwrap();
    println!();
}

fn execute_next_statement(
    program: &Program,
    pc: &mut usize,
    registers: &mut IndexMap<String, usize>,
    count: &mut usize,
) -> ControlFlow<()> {
    execute_statement(&program.statements[*pc - 1], registers, pc);
    *count += 1;

    if *pc > program.statements.len() {
        return ControlFlow::Break(());
    }
    ControlFlow::Continue(())
}

fn print_debug_header(count: &usize, stdout: &mut std::io::Stdout) {
    write!(stdout, "\n\r").unwrap();
    write!(stdout, "URM Debugger (step {})\n\r", count).unwrap();
    stdout
        .execute(PrintStyledContent(
            "============\n\r\n\r".to_string().grey(),
        ))
        .unwrap();
}

fn print_registers_state(registers: &IndexMap<String, usize>, stdout: &mut std::io::Stdout) {
    write!(stdout, "Registers:\n\r").unwrap();
    for (reg, val) in registers {
        stdout
            .execute(PrintStyledContent(format!("{} = {}", reg, val).blue()))
            .unwrap();
        write!(stdout, "\n\r").unwrap();
    }
    write!(stdout, "\n\r").unwrap();
}

fn print_current_instruction(program: &Program, pc: usize, stdout: &mut std::io::Stdout) {
    write!(stdout, "Instruction:\n\r").unwrap();
    stdout
        .execute(PrintStyledContent(
            program.statements[pc - 1].to_string(pc - 1).blue(),
        ))
        .unwrap();
    write!(stdout, "\n\r").unwrap();
}

fn print_tooltip(stdout: &mut std::io::Stdout, debug_state: &DebugMode) {
    write!(stdout, "\n\r").unwrap();
    write!(stdout, "Current mode: ").unwrap();
    match debug_state {
        DebugMode::Auto { timeout } => {
            stdout
                .execute(PrintStyledContent(
                    format!("Auto mode (speed: {} ms per instruction)\n\r", timeout).green(),
                ))
                .unwrap();
            write!(stdout, "- 'm' to switch to manual mode\n\r").unwrap();
            write!(stdout, "- 'j' to decrease speed\n\r").unwrap();
            write!(stdout, "- 'k' to increase speed\n\r").unwrap();
        }
        DebugMode::Manual { step: _ } => {
            stdout
                .execute(PrintStyledContent("Manual mode\n\r".green()))
                .unwrap();
            write!(stdout, "- 'm' to switch to auto mode\n\r").unwrap();
            write!(stdout, "- '<space>' to execute the next instruction\n\r").unwrap();
        }
    }
    write!(stdout, "- 'ESC' to exit\n\r").unwrap();
}

impl DebugMode {
    pub fn handle_key(&mut self, key: KeyCode) {
        match self {
            DebugMode::Auto { timeout } => match key {
                KeyCode::Char('m') => *self = DebugMode::Manual { step: false },
                KeyCode::Char('j') => {
                    *timeout = timeout.saturating_add(100);
                }
                KeyCode::Char('k') => {
                    *timeout = timeout.saturating_sub(100);
                }
                _ => {}
            },
            DebugMode::Manual { step } => match key {
                KeyCode::Char('m') => {
                    *self = DebugMode::Auto {
                        timeout: DEFAULT_TIMEOUT_MILLIS,
                    }
                }
                KeyCode::Char(' ') => {
                    *step = true;
                }
                _ => {}
            },
        }
    }
}

fn get_input(debug_state: &mut DebugMode) {
    let key_polling_timeout = if let DebugMode::Auto { timeout } = debug_state {
        *timeout
    } else {
        100
    };

    if event::poll(Duration::from_millis(key_polling_timeout)).unwrap() {
        if let Event::Key(key_event) = event::read().unwrap() {
            match key_event.code {
                KeyCode::Esc => std::process::exit(0),
                _ => debug_state.handle_key(key_event.code),
            }
        }
    }
}
