fn main() {
    use serde_json;
    use serde_sarif::sarif::Sarif;
    use std::io;
    let sarif: Sarif =
        serde_json::from_reader(io::stdin()).expect("Failed to deserialize Sarif from stdin");

    use std::env;
    let args: Vec<String> = env::args().collect();
    let filename = args.get(1).expect("Failed to read filename argument");

    use std::fs::File;
    let file = File::open(filename).expect("Failed to open previous sarif file");

    let json: String = serde_json::to_string(&sarif).unwrap();
    print!("{json}");
}
