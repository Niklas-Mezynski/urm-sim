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

const DEFAULT_TIMEOUT_MILLIS: u64 = 2000;

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

        if let ControlFlow::Break(_) = get_input(&mut debug_state) {
            break;
        }

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
    stdout.execute(cursor::MoveToNextLine(1)).unwrap();
    stdout.flush().unwrap();
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
    write!(stdout, "URM Debugger (step {})\n\r", count).unwrap();
    stdout
        .execute(PrintStyledContent(
            "=======================\n\r\n\r".to_string().grey(),
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
    // Get n instructions before and after the current instruction (if possible)
    // to print a context around the current instruction
    let context = 2;
    let start = (pc as i32 - context - 1).max(0) as usize;
    let end = (pc + context as usize).min(program.statements.len());

    write!(stdout, "Instructions:\n\r").unwrap();
    for (i, statement) in program.statements[start..end].iter().enumerate() {
        let instr_number = start + i + 1;
        let instr_str = statement.to_string(instr_number);
        if instr_number == pc {
            stdout
                .execute(PrintStyledContent(format!("-> {}", instr_str).blue()))
                .unwrap();
        } else {
            write!(stdout, "   {}", instr_str).unwrap();
        }
        write!(stdout, "\n\r").unwrap();
    }
}

fn print_tooltip(stdout: &mut std::io::Stdout, debug_state: &DebugMode) {
    write!(stdout, "\n\r").unwrap();
    write!(stdout, "Current mode: ").unwrap();
    match debug_state {
        DebugMode::Auto { timeout } => {
            stdout
                .execute(PrintStyledContent(
                    format!(
                        "Auto mode [speed: {} instructions/s ({} ms/instruction)]\n\r",
                        // Round to max 2 decimal places
                        (1000_f64 / (*timeout as f64) * 100.0).round() / 100.0,
                        timeout
                    )
                    .green(),
                ))
                .unwrap();
            write!(stdout, "- 'm' to switch to manual mode\n\r").unwrap();
            write!(stdout, "- 'j' | '↓' to decrease speed\n\r").unwrap();
            write!(stdout, "- 'k' | '↑' to increase speed\n\r").unwrap();
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
                // Decrease speed (increase timeout)
                KeyCode::Char('j') | KeyCode::Down => {
                    let multiplier = f64::log10(*timeout as f64).floor() as u64;

                    let scaling = u64::pow(10, multiplier as u32);

                    *timeout = timeout.saturating_add(scaling).min(100000);
                }
                // Increase speed (decrease timeout)
                KeyCode::Char('k') | KeyCode::Up => {
                    let multiplier =
                        f64::log10(*timeout as f64 - f64::log10(*timeout as f64)).floor() as u64;

                    let scaling = u64::pow(10, multiplier as u32);

                    *timeout = timeout.saturating_sub(scaling).max(1);
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

fn get_input(debug_state: &mut DebugMode) -> ControlFlow<()> {
    let timeout_millis = match debug_state {
        DebugMode::Auto { timeout } => *timeout,
        DebugMode::Manual { step: _ } => 1000,
    };

    if !event::poll(Duration::from_millis(timeout_millis)).unwrap() {
        return ControlFlow::Continue(());
    }

    if let Event::Key(key_event) = event::read().unwrap() {
        match key_event.code {
            KeyCode::Esc => ControlFlow::Break(()),
            KeyCode::Char('c') if key_event.modifiers.contains(event::KeyModifiers::CONTROL) => {
                ControlFlow::Break(())
            }
            code => {
                debug_state.handle_key(code);
                ControlFlow::Continue(())
            }
        }
    } else {
        ControlFlow::Continue(())
    }
}
