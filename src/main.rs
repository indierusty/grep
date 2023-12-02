use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy)]
struct Options {
    /// -r | --recursive
    recursive: bool,
    /// -h | --no-filename
    no_filename: bool,
    /// -v | --invert-match
    invert_match: bool,
}

fn grep<'a>(option: &Options, pattern: &'a str, path: &Path) {
    if path.is_dir() && option.recursive {
        for entry in fs::read_dir(&path).unwrap() {
            let entry = entry.unwrap();
            grep(option, pattern, entry.path().as_path());
        }
    }
    if path.is_file() {
        match fs::read_to_string(&path) {
            Ok(f) => {
                let file: Vec<&str> = f
                    .lines()
                    .filter(|l| {
                        let a = l.contains(pattern);
                        if option.invert_match {
                            !a
                        } else {
                            a
                        }
                    })
                    .collect();

                for l in file {
                    if !option.no_filename {
                        print!("{}:", path.to_str().unwrap());
                    }
                    println!("{}", l);
                }
            }
            Err(e) => eprintln!("ERR: path: {}, err: {}", path.to_str().unwrap(), e),
        }
    }
}

fn main() {
    let mut args = std::env::args().skip(1).peekable();
    let mut pattern = String::new();
    let mut files: Vec<String> = Vec::new();

    let mut options = Options {
        recursive: false,
        no_filename: false,
        invert_match: false,
    };

    let mut pattern_parsed = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-r" | "--recursive" => options.recursive = true,
            "-h" | "--no-filename" => options.no_filename = true,
            "-v" | "--invert-match" => options.invert_match = true,
            _ => {
                // PATTERN comes before FILES hence
                if !pattern_parsed {
                    pattern.push_str(&arg);
                    pattern_parsed = true;
                } else {
                    files.push(arg);
                }
            }
        };
    }

    println!("{:?}", options);

    for f in files {
        let path = Path::new(&f);
        grep(&options, &pattern, &path);
    }
}
