use std::{
    fs::{self, File},
    io::{self, Write},
};

fn main() {
    println!("cargo:rerun-if-changed=src/project.rs");

    let entries = fs::read_dir("./fixtures")
        .expect("failed to list fixture files")
        .map(|res| res.map(|x| x.path()))
        .collect::<Result<Vec<_>, io::Error>>()
        .unwrap();

    let mut entire_string = String::new();
    entire_string.push_str("[\n");

    for (index, fixture_path) in entries.iter().enumerate() {
        println!("Fixture file: {:?}", fixture_path);
        match fs::read_to_string(fixture_path) {
            Ok(d) => {
                entire_string.push_str(&d);
                if index < (entries.len() - 1) {
                    entire_string.push(',');
                }
            }
            Err(e) => {
                panic!(
                    "Something went wrong reading the contents of the fixture file: {}",
                    e
                );
            }
        }
    }

    entire_string.push_str("\n]");

    let mut f = File::create("src/all_fixtures.json").expect("failed to create output file");
    f.write_all(entire_string.as_bytes())
        .expect("failed to write");
}
