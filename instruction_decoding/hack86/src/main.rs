mod decode;
mod simulate;

fn main() {
    let args = std::env::args().collect::<Vec<String>>();

    let simulation_mode = args.iter().any(|s| s == "-s");
    let dump = args.iter().any(|s| s == "-d");

    let path = args.iter().skip(1).find(|s| !s.starts_with("-"));

    if let Some(path) = path {
        let instruction_stream = std::fs::read(&path).expect("failed to read file");
        if simulation_mode {
            simulate(instruction_stream, dump);
        } else {
            println!("bits 16");
            println!("; {} disassembly:", &path);
            decode_and_print(&instruction_stream);
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
            instruction_index += usize::from(instruction.length);
        } else {
            panic!(
                "unsupported instruction {:#010b} at offset {}",
                instruction_stream[instruction_index], instruction_index
            );
        }
    }
}

fn simulate(instructions: Vec<u8>, dump: bool) {
    let mut computer = simulate::Hack86::new(instructions);
    computer.simulate();
    if dump {
        let memory = computer.memory();
        std::fs::write("hack86_memory.data", memory).expect("Failed to write memory to file");
    }
}
