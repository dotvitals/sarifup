fn main() {
    use serde_json;
    use serde_sarif::sarif::Sarif;
    use std::io;
    let stdin_sarif: Sarif =
        serde_json::from_reader(io::stdin()).expect("Error deserializing stdin SARIF");

    use std::env;
    let args: Vec<String> = env::args().collect();
    let filename = args.get(1).expect("Error getting SARIF filename argument");

    use std::fs::File;
    let file = File::open(filename).expect("Error opening SARIF file");
    let file_sarif: Sarif = serde_json::from_reader(file).expect("Error deserializing SARIF file");

    let json: String = serde_json::to_string(&stdin_sarif).unwrap();
    print!("{json}");
}
