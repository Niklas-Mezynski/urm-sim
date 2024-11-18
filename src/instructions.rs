#[derive(Debug)]
pub enum Condition {
    Equal,
    NotEqual,
}

#[derive(Debug)]
pub enum Statement {
    ConditionalGoto {
        register: String,
        condition: Condition,
        target: usize,
    },
    Increment {
        register: String,
    },
    Decrement {
        register: String,
    },
    ZeroAssignment {
        register: String,
    },
    Goto {
        target: usize,
    },
}

impl Statement {
    pub fn to_string(&self, instr_number: usize) -> String {
        match self {
            Statement::ConditionalGoto {
                register,
                condition,
                target,
            } => format!(
                "{}: if {} {} 0 goto {};",
                instr_number,
                register,
                match condition {
                    Condition::Equal => "==",
                    Condition::NotEqual => "!=",
                },
                target
            ),
            Statement::Increment { register } => format!("{}: {}++;", instr_number, register),
            Statement::Decrement { register } => format!("{}: {}--;", instr_number, register),
            Statement::ZeroAssignment { register } => {
                format!("{}: {} = 0;", instr_number, register)
            }
            Statement::Goto { target } => format!("{}: goto {};", instr_number, target),
        }
    }
}

#[derive(Debug)]
pub struct Program {
    pub input_registers: Vec<String>,
    pub statements: Vec<Statement>,
    pub output_register: String,
}
