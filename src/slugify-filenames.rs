extern crate clap;
use clap::{App, Arg, ArgMatches};
use console::style;
use glob::glob_with;
use glob::MatchOptions;
use regex::Regex;
use std::borrow::Borrow;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct SlugifyOptions<'a> {
    verbose: bool,
    recursive: bool,
    case_sensitive: bool,
    skip_dirs: bool,
    should_lower_the_case: bool,
    ignore_hidden: bool,
    silent: bool,
    dry_run: bool,
    targets: Vec<&'a str>,
}

impl SlugifyOptions<'_> {
    pub fn from_matches<'a>(matches: &'a ArgMatches) -> SlugifyOptions<'a> {
        let recursive = matches.is_present("recursive");
        let verbose = matches.is_present("verbose");
        let silent = matches.is_present("silent");
        let dry_run = matches.is_present("dry_run");
        let case_sensitive = !matches.is_present("case_insensitive");
        let skip_dirs = !matches.is_present("case_insensitive");
        let should_lower_the_case = !matches.is_present("should_lower_the_case");
        let ignore_hidden = !matches.is_present("include_hidden");
        let targets: Vec<_> = matches.values_of("target").unwrap().collect();

        SlugifyOptions {
            recursive,
            verbose,
            silent,
            dry_run,
            targets,
            ignore_hidden,
            case_sensitive,
            skip_dirs,
            should_lower_the_case,
        }
    }
    pub fn targets<'a>(&'a self) -> Vec<&'a str> {
        self.targets.iter().map(|t| t.to_owned()).collect()
    }
    pub fn case_sensitive(&self) -> bool {
        self.case_sensitive
    }
    pub fn should_lower_the_case(&self) -> bool {
        self.should_lower_the_case
    }
    pub fn skip_dirs(&self) -> bool {
        self.skip_dirs
    }
    pub fn silent(&self) -> bool {
        self.silent
    }
    pub fn verbose(&self) -> bool {
        self.verbose
    }
    pub fn ignore_hidden(&self) -> bool {
        self.ignore_hidden
    }
    pub fn dry_run(&self) -> bool {
        self.dry_run
    }
    pub fn recursive(&self) -> bool {
        self.recursive
    }
}

fn is_blacklisted(path: &str) -> bool {
    let blacklisted_names: Vec<&'static str> = Vec::from([
        "__pycache__",
        "node_modules",
        ".venv",
        "target",
        ".bento",
        "testsuite",
    ]);
    if blacklisted_names.contains(&path) {
        return true;
    }
    // let blacklisted_extensions: Vec<&'static str> = Vec::from(["rs", "rb", "py"]);
    // for forbidden in blacklisted_extensions {
    //     if path.ends_with(forbidden) {
    //         return true;
    //     }
    // }
    return false;
}

fn main() {
    let app = App::new("slugify-filenames")
        .version("1.0")
        .author("Gabriel Falcão <gabriel@nacaolivre.org>")
        .about("slugify filenames")
        .arg(
            Arg::with_name("target")
                .help("a glob pattern")
                .default_value("*")
                .multiple(true)
                .required(false),
        )
        .arg(
            Arg::with_name("case_insensitive")
                .help("match glob case insensitive")
                .long("case-insensitive")
                .short("i")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("skip_dirs")
                .help("when in recursive mode, this flag switches on the ability to enter directories for slugifying its contents without slugifying the directory itself.")
                .long("skip-dirs")
                .short("D")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("recursive")
                .help("recurse directories")
                .long("recursive")
                .short("r")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("silent")
                .long("silent")
                .short("s")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("verbose")
                .long("verbose")
                .short("v")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("dry_run")
                .long("dry-run")
                .short("n")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("should_lower_the_case")
                .long("force-lowercase")
                .short("l")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("include_hidden")
                .help("include hidden files")
                .long("include-hidden")
                .short("D")
                .takes_value(false),
        );

    let matches = app.get_matches();
    let slugify_options = SlugifyOptions::from_matches(&matches);
    for target in slugify_options.targets() {
        // let target = match target.len() == 0 {
        //     true => target,
        //     false => "*"
        // };
        let glob_options = MatchOptions {
            case_sensitive: slugify_options.case_sensitive(),
            require_literal_separator: true,
            require_literal_leading_dot: true,
        };
        let entries = glob_with(target, glob_options).unwrap();
        for entry in entries {
            if let Ok(path) = entry {
                slugify_filenames(path, &slugify_options);
            }
        }
    }
}

fn get_source_info<'a>(path: &'a PathBuf) -> (PathBuf, String, String) {
    let parent = match path.parent() {
        Some(parent) => parent,
        None => path.as_path(),
    };
    let extension = path
        .extension()
        .unwrap_or(OsStr::new(""))
        .to_str()
        .unwrap()
        .to_string();
    let source = match path.file_name() {
        Some(path) => path.to_str().unwrap().to_string(),
        None => match parent.file_name() {
            Some(path) => path.to_str().unwrap().to_string(),
            None => String::new(),
        },
    };

    (parent.to_path_buf(), source.to_string(), extension)
}

fn slugify_string<'a>(value: &'a str, repchar: &'a str, options: &SlugifyOptions) -> String {
    let corners = format!("(^{}+|{}+$)", repchar, repchar);
    let re = Regex::new(r"[^a-zA-Z0-9_-]+").unwrap();
    let value = re.replace_all(value, repchar);
    let re = Regex::new(corners.as_str()).unwrap();
    let value = re.replace_all(value.borrow(), "");

    if options.should_lower_the_case() {
        value.to_string()
    } else {
        value.to_lowercase().to_string()
    }
}

fn slugify_filenames(path: PathBuf, options: &SlugifyOptions) {
    let (parent, original, extension) = get_source_info(&path);

    if options.ignore_hidden() && original.starts_with(".") {
        if options.verbose() {
            println!("{}", style(original).color256(237));
        }
        return;
    }
    if options.skip_dirs() && path.is_dir() {
        return;
    }
    if is_blacklisted(original.as_str()) {
        if options.verbose() {
            println!("{}", style(original).color256(237));
        }
        return;
    }

    let has_extension = extension.len() > 0;
    let name = match has_extension {
        true => {
            let re = Regex::new(format!(".{}$", extension).as_str()).expect("invalid regex");
            let name = re.replace(&original, "");
            name.to_string()
        }
        false => String::from(original.as_str()),
    };
    let slugified = slugify_string(name.as_str(), "-", options);
    let target = match has_extension {
        true => format!("{}.{}", slugified, extension),
        false => slugified,
    };

    let source = parent.join(original);
    let destination = parent.join(target);

    if options.recursive() && path.is_dir() {
        for entry in path.read_dir().expect("failed to list directory") {
            if let Ok(entry) = entry {
                slugify_filenames(entry.path(), options);
            }
        }
    }

    let source = source.to_str().unwrap();
    let destination = destination.to_str().unwrap();

    if source != destination {
        if options.dry_run() {
            println!(
                "would rename {} to {}",
                style(source).color256(220),
                style(
                    Path::new(destination)
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                )
                .color256(184),
            );
        } else {
            if let Err(error) = fs::rename(source, destination) {
                println!(
                    "{} {}: {} -> {}",
                    style("Error").color256(195),
                    style(error.to_string()).color256(220),
                    source,
                    destination
                );
            } else {
                if !options.silent() {
                    println!(
                        "{} -> {}",
                        style(source).color256(45),
                        style(destination).color256(49),
                    );
                }
            }
        }
    } else {
        if options.verbose() {
            println!("{}", style(destination).color256(220));
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{get_source_info, slugify_string};
    use std::fs;

    use k9::assert_equal;
    use std::path::Path;

    fn stub_options() {
        SlugifyOptions {
           r#false,
           r#false,
           r#false,
           r#false,
           r#false,
           r#false,
           r#false,
           r#false,
        }

    }
    #[test]
    fn test_slugify_string_basic_scenario() {
        assert_equal!(
            "this-is-a-basic-string-123",
            slugify_string(" This is a basic string: 123 ", "-"),
        );
    }
    #[test]
    fn test_slugify_string_special_chars_scenario1() {
        assert_equal!(
            "test-this-is-a-special-string-never-expected-123",
            slugify_string(
                "[test]^This is a special string @&^,never{expected}: 123 &&^^ ",
                "-"
            )
        );
    }

    #[test]
    fn test_get_source_info() {
        fs::create_dir_all("tmp/bar").unwrap();
        fs::write("tmp/bar/some-file.txt", b"dummy").unwrap();

        let path = Path::new("tmp/bar/some-file.txt").to_path_buf();

        let (parent, source, extension) = get_source_info(&path);

        assert_equal!("tmp/bar", parent.as_os_str().to_str().unwrap());
        assert_equal!("some-file.txt", source);
        assert_equal!("txt", extension);
    }
}
