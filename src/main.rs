use std::{
    env,
    error::Error,
    fs::{self, File},
    process,
};

use text_aligner::{run, Config, FileWriter, StdoutWriter};

fn main() -> Result<(), Box<dyn Error>> {
    let args = env::args().collect::<Vec<String>>();

    let Config {
        destination_path,
        len,
        file_path,
        align,
    } = Config::build(&args).unwrap_or_else(|err| {
        eprintln!("Problem parsing parameters: {err}");
        process::exit(1);
    });

    match destination_path {
        Some(path) => {
            run(
                &fs::read_to_string(file_path)?,
                &mut FileWriter {
                    file: File::create(path)?,
                },
                len,
                &align,
            )?;
        }
        None => {
            run(
                &fs::read_to_string(file_path)?,
                &mut StdoutWriter,
                len,
                &align,
            )?;
        }
    }

    Ok(())
}
