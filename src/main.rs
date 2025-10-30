fn main() -> Result<(), serde_json::Error> {
    use serde_json;
    use serde_json::Value;
    use std::io;
    let json: Value = serde_json::from_reader(io::stdin())?;
    print!("{json}");
    Ok(())
}
