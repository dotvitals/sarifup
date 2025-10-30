use std::io;

fn main() -> io::Result<()> {
    let stdin = io::read_to_string(io::stdin())?;
    print!("{stdin}");
    Ok(())
}
