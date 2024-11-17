use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "urm.pest"] // This is the path to the grammar file
struct URMParser;

use crate::instructions::*;

pub fn parse_urm_code(input: &str) -> Result<Program, String> {
    // Parse the input using the Pest parser
    let parsed =
        URMParser::parse(Rule::program, input).map_err(|e| format!("Parsing error: {}", e))?;

    // Check if the iterator has only one top-level element (Rule::program)
    let program_pair = parsed
        .into_iter()
        .next()
        .ok_or_else(|| "Parsing failed: no program rule found".to_string())?;

    if program_pair.as_rule() != Rule::program {
        return Err("Parsing error: expected program rule".to_string());
    }

    let mut input_registers = Vec::new();
    let mut statements = Vec::new();
    let mut output_register = None;

    for pair in program_pair.into_inner() {
        match pair.as_rule() {
            Rule::input_decl => {
                input_registers = pair.into_inner().map(|r| r.as_str().to_string()).collect();
            }
            Rule::statement => {
                for inner_pair in pair.into_inner() {
                    match inner_pair.as_rule() {
                        Rule::increment => {
                            let register =
                                inner_pair.into_inner().next().unwrap().as_str().to_string();
                            statements.push(Statement::Increment { register });
                        }
                        Rule::decrement => {
                            let register =
                                inner_pair.into_inner().next().unwrap().as_str().to_string();
                            statements.push(Statement::Decrement { register });
                        }
                        Rule::reset => {
                            let register =
                                inner_pair.into_inner().next().unwrap().as_str().to_string();
                            statements.push(Statement::ZeroAssignment { register });
                        }
                        Rule::conditional_eq => {
                            let mut parts = inner_pair.into_inner();
                            let register = parts.next().unwrap().as_str().to_string();
                            let target = parts.next().unwrap().as_str().parse::<usize>().unwrap();
                            statements.push(Statement::ConditionalGoto {
                                register,
                                condition: Condition::Equal,
                                target,
                            });
                        }
                        Rule::conditional_neq => {
                            let mut parts = inner_pair.into_inner();
                            let register = parts.next().unwrap().as_str().to_string();
                            let target = parts.next().unwrap().as_str().parse::<usize>().unwrap();
                            statements.push(Statement::ConditionalGoto {
                                register,
                                condition: Condition::NotEqual,
                                target,
                            });
                        }
                        Rule::goto => {
                            let target = inner_pair
                                .into_inner()
                                .next()
                                .unwrap()
                                .as_str()
                                .parse::<usize>()
                                .unwrap();
                            statements.push(Statement::Goto { target });
                        }
                        _ => unreachable!("Unexpected statement rule: {:?}", inner_pair),
                    }
                }
            }
            Rule::output_decl => {
                output_register = Some(pair.into_inner().next().unwrap().as_str().to_string());
            }
            _ => unreachable!("Unexpected top level rule: {:?}", pair),
        }
    }

    if let Some(output) = output_register {
        Ok(Program {
            input_registers,
            statements,
            output_register: output,
        })
    } else {
        Err("No output register found".to_string())
    }
}
