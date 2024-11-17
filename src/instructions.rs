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

#[derive(Debug)]
pub struct Program {
    pub input_registers: Vec<String>,
    pub statements: Vec<Statement>,
    pub output_register: String,
}
