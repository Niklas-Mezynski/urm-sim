mod instructions;
mod parser;
mod simulator;

fn main() {
    let urm_code = r#"
        in(R1, R2)
        if R2 == 0 goto 5;
        R2--;
        R1++;
        goto 1;
        out(R1)
        "#;

    let program = match parser::parse_urm_code(urm_code) {
        Ok(program) => program,
        Err(e) => {
            eprintln!("Failed to parse: {}", e);
            std::process::exit(1);
        }
    };

    let program_result = simulator::simulate_urm(&program, vec![5, 3]);

    println!("Program result: {}", program_result);
}
