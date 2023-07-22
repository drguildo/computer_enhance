mod decode;
mod simulate;

fn main() {
    let mut simulation_mode: bool = false;

    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Some(first) = args.first() {
        let path = if first == "-s" {
            simulation_mode = true;

            args.iter().nth(1)
        } else {
            args.first()
        };

        if let Some(path) = path {
            let instruction_stream = std::fs::read(path).expect("failed to read file");
            if simulation_mode {
                simulate(&instruction_stream);
            } else {
                decode_and_print(&instruction_stream);
            }
        }
    } else {
        eprintln!("Please specify the path to a binary.");
    }
}

fn decode_and_print(instruction_stream: &[u8]) {
    let mut instruction_index = 0;
    while instruction_index < instruction_stream.len() {
        if let Ok(instruction) =
            decode::decode_instruction(&instruction_stream[instruction_index..])
        {
            println!("{}", instruction.instruction_category);
            instruction_index += instruction.length;
        } else {
            panic!(
                "unsupported instruction {:#010b} at offset {}",
                instruction_stream[instruction_index], instruction_index
            );
        }
    }
}

fn simulate(instruction_stream: &[u8]) {
    let mut registers = simulate::Registers::new();

    let mut instruction_index = 0;
    while instruction_index < instruction_stream.len() {
        if let Ok(instruction) =
            decode::decode_instruction(&instruction_stream[instruction_index..])
        {
            registers.simulate(&instruction);
            instruction_index += instruction.length;
        } else {
            panic!(
                "unsupported instruction {:#010b} at offset {}",
                instruction_stream[instruction_index], instruction_index
            );
        }
    }

    println!();
    println!("Final registers:");
    println!("{}", registers);
}
