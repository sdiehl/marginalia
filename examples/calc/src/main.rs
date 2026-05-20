use std::{
    env, fs,
    io::{self, Read, Write},
    path::PathBuf,
    process::ExitCode,
};

use calc::format_source;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let (write_in_place, path) = parse_args(&args);

    let source = if let Some(p) = &path {
        match fs::read_to_string(p) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("calcfmt: cannot read {}: {e}", p.display());
                return ExitCode::from(2);
            }
        }
    } else {
        let mut buf = String::new();
        if let Err(e) = io::stdin().read_to_string(&mut buf) {
            eprintln!("calcfmt: cannot read stdin: {e}");
            return ExitCode::from(2);
        }
        buf
    };

    let formatted = match format_source(&source) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("calcfmt: {e}");
            return ExitCode::FAILURE;
        }
    };

    if write_in_place {
        if let Some(p) = path {
            if let Err(e) = fs::write(&p, &formatted) {
                eprintln!("calcfmt: cannot write {}: {e}", p.display());
                return ExitCode::from(2);
            }
            return ExitCode::SUCCESS;
        }
        eprintln!("calcfmt: --write requires a file path");
        return ExitCode::from(2);
    }

    let mut stdout = io::stdout().lock();
    if stdout.write_all(formatted.as_bytes()).is_err() {
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn parse_args(args: &[String]) -> (bool, Option<PathBuf>) {
    let mut write = false;
    let mut path: Option<PathBuf> = None;
    for a in args {
        match a.as_str() {
            "-w" | "--write" => write = true,
            other => path = Some(PathBuf::from(other)),
        }
    }
    (write, path)
}
