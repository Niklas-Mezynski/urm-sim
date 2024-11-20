use crate::{instructions::Program, simulator::execute_statement};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    style::{PrintStyledContent, Stylize},
    terminal::{self, disable_raw_mode, enable_raw_mode, Clear, ClearType},
    ExecutableCommand, QueueableCommand,
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

pub fn run_with_debug(
    program: &Program,
    registers: &mut IndexMap<String, usize>,
    pc: &mut usize,
) -> std::io::Result<()> {
    let mut count = 1;
    let mut last_execution = std::time::Instant::now();
    let mut debug_state = DebugMode::Manual { step: false };

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = stdout();

    // Use alternate screen buffer to avoid corrupting the main terminal
    execute!(
        stdout,
        terminal::EnterAlternateScreen,
        cursor::Hide,
        Clear(ClearType::All)
    )?;

    loop {
        // Buffer all drawing operations
        stdout
            .queue(cursor::MoveTo(0, 0))?
            .queue(Clear(ClearType::All))?;

        // Draw UI elements
        draw_debug_header(&count, &mut stdout)?;
        draw_registers_state(registers, &mut stdout)?;
        draw_current_instruction(program, *pc, &mut stdout)?;
        draw_tooltip(&mut stdout, &debug_state)?;

        // Flush all buffered operations at once
        stdout.flush()?;

        // Process debug state
        match debug_state {
            DebugMode::Auto { timeout } => {
                if last_execution.elapsed() >= Duration::from_millis(timeout) {
                    last_execution = std::time::Instant::now();
                    if let ControlFlow::Break(_) =
                        execute_next_statement(program, pc, registers, &mut count)
                    {
                        break;
                    }
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

        // Handle input with proper event filtering
        if let ControlFlow::Break(_) = handle_input(&mut debug_state)? {
            break;
        }
    }

    // Cleanup
    execute!(stdout, cursor::Show, terminal::LeaveAlternateScreen)?;
    disable_raw_mode()?;

    Ok(())
}

fn handle_input(debug_state: &mut DebugMode) -> std::io::Result<ControlFlow<()>> {
    let timeout_millis = match debug_state {
        DebugMode::Auto { timeout } => *timeout,
        DebugMode::Manual { .. } => 100, // Shorter polling interval for better responsiveness
    };

    if !event::poll(Duration::from_millis(timeout_millis))? {
        return Ok(ControlFlow::Continue(()));
    }

    if let Event::Key(key_event) = event::read()? {
        // Only handle key press events, ignore releases
        if key_event.kind == KeyEventKind::Press {
            match key_event.code {
                KeyCode::Esc => Ok(ControlFlow::Break(())),
                KeyCode::Char('c')
                    if key_event.modifiers.contains(event::KeyModifiers::CONTROL) =>
                {
                    Ok(ControlFlow::Break(()))
                }
                code => {
                    debug_state.handle_key(code);
                    Ok(ControlFlow::Continue(()))
                }
            }
        } else {
            Ok(ControlFlow::Continue(()))
        }
    } else {
        Ok(ControlFlow::Continue(()))
    }
}

// Drawing functions remain similar but with proper error handling
fn draw_debug_header(count: &usize, stdout: &mut std::io::Stdout) -> std::io::Result<()> {
    write!(stdout, "URM Debugger (step {})\r\n", count)?;
    stdout.queue(PrintStyledContent(
        "=======================\r\n\r\n".to_string().grey(),
    ))?;
    Ok(())
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

fn draw_registers_state(
    registers: &IndexMap<String, usize>,
    stdout: &mut std::io::Stdout,
) -> std::io::Result<()> {
    stdout.queue(PrintStyledContent("Registers:\r\n".grey()))?;

    for (reg, val) in registers {
        stdout.queue(PrintStyledContent(format!("{} = {}\r\n", reg, val).blue()))?;
    }

    stdout.queue(PrintStyledContent("\r\n".grey()))?;

    Ok(())
}

fn draw_current_instruction(
    program: &Program,
    pc: usize,
    stdout: &mut std::io::Stdout,
) -> std::io::Result<()> {
    // Context of instructions around current program counter
    let context = 2;
    let start = (pc as i32 - context - 1).max(0) as usize;
    let end = (pc + context as usize).min(program.statements.len());

    // write!(stdout, "Instructions:\r\n")?;
    stdout.queue(PrintStyledContent("Instructions:\r\n".grey()))?;

    for (i, statement) in program.statements[start..end].iter().enumerate() {
        let instr_number = start + i + 1;
        let instr_str = statement.to_string(instr_number);

        if instr_number == pc {
            // Highlight current instruction
            stdout.queue(PrintStyledContent(format!("-> {}\r\n", instr_str).blue()))?;
        } else {
            stdout.queue(PrintStyledContent(format!("   {}\r\n", instr_str).grey()))?;
        }
    }

    stdout.queue(PrintStyledContent("\r\n".white()))?;

    Ok(())
}

fn draw_tooltip(stdout: &mut std::io::Stdout, debug_state: &DebugMode) -> std::io::Result<()> {
    stdout.queue(PrintStyledContent("Controls: ".grey()))?;

    match debug_state {
        DebugMode::Auto { timeout } => {
            // Auto mode tooltip
            stdout.queue(PrintStyledContent(
                format!(
                    "Auto mode [speed: {} instructions/s ({} ms/instruction)]\n\r",
                    // Round to max 2 decimal places
                    (1000_f64 / (*timeout as f64) * 100.0).round() / 100.0,
                    timeout
                )
                .green(),
            ))?;

            stdout.queue(PrintStyledContent(
                "- 'm': Switch to Manual Mode\r\n\
                 - '↓'/'j': Decrease Speed\r\n\
                 - '↑'/'k': Increase Speed\r\n"
                    .grey(),
            ))?;
        }
        DebugMode::Manual { .. } => {
            // Manual mode tooltip
            stdout.queue(PrintStyledContent("Manual Mode\r\n".green()))?;

            stdout.queue(PrintStyledContent(
                "- 'm': Switch to Auto Mode\r\n\
                 - 'Space': Execute Next Instruction\r\n"
                    .grey(),
            ))?;
        }
    }

    // Common controls
    stdout.queue(PrintStyledContent(
        "- 'Esc'/'Ctrl+c': Exit Debugger\r\n".grey(),
    ))?;

    Ok(())
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
