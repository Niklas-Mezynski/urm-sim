use crate::{instructions::Program, simulator::execute_statement};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use indexmap::IndexMap;
use ratatui::{
    prelude::*,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use std::{
    io,
    time::{Duration, Instant},
};

const DEFAULT_TIMEOUT_MILLIS: u64 = 2000;

#[derive(Debug)]
pub enum DebugMode {
    Auto { timeout: u64 },
    Manual { step: bool },
}

pub struct DebuggerState {
    debug_mode: DebugMode,
    instruction_count: usize,
    last_execution: Instant,
}

impl DebuggerState {
    fn execute_next_instruction(
        &mut self,
        program: &Program,
        registers: &mut IndexMap<String, usize>,
        pc: &mut usize,
    ) -> bool {
        execute_statement(&program.statements[*pc - 1], registers, pc);
        self.instruction_count += 1;
        *pc > program.statements.len()
    }
}

pub fn run_with_debug(
    program: &Program,
    registers: &mut IndexMap<String, usize>,
    pc: &mut usize,
) -> io::Result<()> {
    // Terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // State management
    let mut debugger_state = DebuggerState {
        debug_mode: DebugMode::Manual { step: false },
        instruction_count: 1,
        last_execution: Instant::now(),
    };

    // Main loop
    loop {
        // Render the UI
        terminal.draw(|frame| ui(frame, &debugger_state, program, registers, pc))?;

        // Handle debug mode execution
        match debugger_state.debug_mode {
            DebugMode::Auto { timeout } => {
                if debugger_state.last_execution.elapsed() >= Duration::from_millis(timeout) {
                    debugger_state.last_execution = Instant::now();
                    if debugger_state.execute_next_instruction(program, registers, pc) {
                        break;
                    }
                }
            }
            DebugMode::Manual { step } => {
                if step {
                    debugger_state.debug_mode = DebugMode::Manual { step: false };
                    if debugger_state.execute_next_instruction(program, registers, pc) {
                        break;
                    }
                }
            }
        }

        // Handle input
        if handle_input(&mut debugger_state)? {
            break;
        }
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    Ok(())
}

fn handle_input(state: &mut DebuggerState) -> io::Result<bool> {
    let timeout_millis = match state.debug_mode {
        DebugMode::Auto { timeout } => timeout,
        DebugMode::Manual { .. } => 100,
    };

    if !event::poll(Duration::from_millis(timeout_millis))? {
        return Ok(false);
    }

    if let Event::Key(key_event) = event::read()? {
        if key_event.kind == KeyEventKind::Press {
            match key_event.code {
                KeyCode::Esc | KeyCode::Char('q') => return Ok(true),
                KeyCode::Char('c')
                    if key_event.modifiers.contains(event::KeyModifiers::CONTROL) =>
                {
                    return Ok(true)
                }
                code => state.debug_mode.handle_key(code),
            }
        }
    }

    Ok(false)
}

fn ui(
    frame: &mut Frame,
    state: &DebuggerState,
    program: &Program,
    registers: &IndexMap<String, usize>,
    pc: &usize,
) {
    let layout = Layout::new(
        Direction::Vertical,
        [
            Constraint::Length(3), // Header
            Constraint::Min(10),   // Main content
            Constraint::Length(5), // Footer/Controls
        ],
    )
    .split(frame.area());

    // Header
    let header = Paragraph::new(format!("URM Debugger (step {})", state.instruction_count))
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(header, layout[0]);

    // Main content (split into two columns)
    let content_layout = Layout::new(
        Direction::Horizontal,
        [
            Constraint::Percentage(40), // Registers
            Constraint::Percentage(60), // Instructions
        ],
    )
    .split(layout[1]);

    // Registers
    let registers_block = Block::default().title("Registers").borders(Borders::ALL);
    let registers_content: Vec<Line> = registers
        .iter()
        .map(|(reg, val)| Line::from(format!("{} = {}", reg, val)))
        .collect();
    let registers_paragraph = Paragraph::new(registers_content)
        .block(registers_block)
        .wrap(Wrap { trim: true });
    frame.render_widget(registers_paragraph, content_layout[0]);

    // Instructions
    let instructions_block = Block::default().title("Instructions").borders(Borders::ALL);

    let available_rows = content_layout[0].as_size();
    let context: i32 = (available_rows.height as i32 - 2) / 2;
    let start = (*pc as i32 - context - 1).max(0) as usize;
    let end = (*pc + context as usize).min(program.statements.len());

    let instruction_content: Vec<Line> = program.statements[start..end]
        .iter()
        .enumerate()
        .map(|(i, statement)| {
            let instr_number = start + i + 1;
            let instr_str = statement.to_string(instr_number);

            if instr_number == *pc {
                Line::from(format!("-> {}", instr_str)).style(
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Line::from(format!("   {}", instr_str))
            }
        })
        .collect();

    let instructions_paragraph = Paragraph::new(instruction_content)
        .block(instructions_block)
        .wrap(Wrap { trim: true });
    frame.render_widget(instructions_paragraph, content_layout[1]);

    // Footer/Controls
    let controls_text = match state.debug_mode {
        DebugMode::Auto { timeout } => format!(
            "Mode: Auto (Speed: {:.2} inst/s) | 'm': Manual | '↓'/'j': Slower | '↑'/'k': Faster",
            1000.0 / timeout as f64
        ),
        DebugMode::Manual { .. } => {
            "Mode: Manual | 'm': Auto | 'Space': Next Instruction".to_string()
        }
    };

    let controls = Paragraph::new(controls_text)
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::TOP));
    frame.render_widget(controls, layout[2]);
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
