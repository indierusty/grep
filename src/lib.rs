use regex::Regex;
use std::fs;
use std::io::{stdin, stdout, Write};
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

impl Options {
    fn new(recursive: bool, no_filename: bool, invert_match: bool) -> Self {
        Self {
            recursive,
            no_filename,
            invert_match,
        }
    }
}

impl Default for Options {
    fn default() -> Self {
        Self::new(false, false, false)
    }
}

pub fn run() {
    let (pattern, mut files, options) = parse_arg();

    if pattern.is_empty() {
        // print HELP
        return;
    }

    if files.is_empty() {
        files.push("-".to_string())
    }

    // TODO: Handle Err
    let regex = regex::Regex::new(&pattern).unwrap();

    for f in files {
        if &f == "-" {
            for line in stdin().lines() {
                let src = grep(&options, &regex, &line.unwrap(), "(stdin)");
                print(&src);
            }
        }
        let path = Path::new(&f);
        recurse(&options, &regex, &path);
    }
}

fn grep<'a>(option: &Options, regex: &Regex, f: &str, file_name: &str) -> String {
    let file: Vec<&str> = f
        .lines()
        .filter(|l| {
            let a = regex.is_match(l);
            // let a = l.contains(pattern);
            if option.invert_match {
                !a
            } else {
                a
            }
        })
        .collect();

    let mut output = String::new();

    for l in file {
        if !option.no_filename {
            output.push_str(file_name);
            output.push(':');
        }
        output.push_str(l);
        output.push('\n');
    }

    output
}

fn parse_arg() -> (String, Vec<String>, Options) {
    let mut args = std::env::args().skip(1).peekable();
    let mut pattern = String::new();
    let mut files: Vec<String> = Vec::new();

    let mut options = Options::default();

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

    (pattern, files, options)
}

fn print(src: &str) {
    stdout().write_all(src.as_bytes()).unwrap();
}

fn recurse<'a>(option: &Options, regex: &Regex, path: &Path) {
    if path.is_dir() && option.recursive {
        for entry in fs::read_dir(&path).unwrap() {
            let entry = entry.unwrap();
            recurse(option, regex, entry.path().as_path());
        }
    }
    if path.is_file() {
        match fs::read_to_string(&path) {
            Ok(f) => {
                let src = grep(option, regex, f.as_str(), path.to_str().unwrap());
                print(&src);
            }
            Err(e) => eprintln!("ERR: path: {}, err: {}", path.to_str().unwrap(), e),
        }
    }
}

#[cfg(test)]
mod test {
    use super::{grep, Options};

    const INPUT: &str = concat!(
        "abc def\n",
        "ijk lmn opq\n",
        "hello123\n",
        "1234 xyz\n",
        "a b c def\n"
    );

    #[test]
    fn basic() {
        let regex = regex::Regex::new("abc");
        let g = grep(&Options::default(), &regex.unwrap(), INPUT, "text");

        let left: Vec<&str> = g.lines().collect();
        let right = ["text:abc def"];

        assert_eq!(left, right);
    }

    #[test]
    fn options() {
        let regex = regex::Regex::new("def");
        let g = grep(
            &Options::new(false, true, false),
            &regex.unwrap(),
            INPUT,
            "text",
        );

        let left: Vec<&str> = g.lines().collect();
        let right = ["abc def", "a b c def"];

        assert_eq!(left, right);
    }

    #[test]
    fn invert() {
        let regex = regex::Regex::new("^a");
        let g = grep(
            &Options::new(false, true, true),
            &regex.unwrap(),
            INPUT,
            "text",
        );

        let left: Vec<&str> = g.lines().collect();
        let right = ["ijk lmn opq", "hello123", "1234 xyz"];
        assert_eq!(left, right);
    }
}
