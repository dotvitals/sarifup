fn main() -> Result<(), serde_json::Error> {
    use serde_json;
    use serde_sarif::sarif::Sarif;
    use std::io;
    let sarif: Sarif = serde_json::from_reader(io::stdin())?;

    use std::env;
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("Please provide previous sarif filename.");
    }

    let json: String = serde_json::to_string(&sarif).unwrap();
    print!("{json}");
    Ok(())
}
