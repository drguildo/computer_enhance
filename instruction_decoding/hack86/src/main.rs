mod decode;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let path = args.first().expect("failed to get path to binary");

    let instruction_stream = std::fs::read(path).expect("failed to read file");
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
