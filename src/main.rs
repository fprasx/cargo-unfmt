use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

use anstyle::*;
use anyhow::Context;
use clap::Arg;
use regex::bytes::Regex;
use walkdir::WalkDir;

// This is the theme cargo uses as of
// https://github.com/rust-lang/cargo/commit/a59aba136aab5510c16b0750a36cbd9916f91796
pub const HEADER: Style = AnsiColor::Green.on_default().effects(Effects::BOLD);
pub const USAGE: Style = AnsiColor::Green.on_default().effects(Effects::BOLD);
pub const LITERAL: Style = AnsiColor::Cyan.on_default().effects(Effects::BOLD);
pub const PLACEHOLDER: Style = AnsiColor::Cyan.on_default();
pub const ERROR: Style = AnsiColor::Red.on_default().effects(Effects::BOLD);
pub const VALID: Style = AnsiColor::Cyan.on_default().effects(Effects::BOLD);
pub const INVALID: Style = AnsiColor::Yellow.on_default().effects(Effects::BOLD);

fn main() -> anyhow::Result<()> {
    let styles = {
        clap::builder::styling::Styles::styled()
            .header(HEADER)
            .usage(USAGE)
            .literal(LITERAL)
            .placeholder(PLACEHOLDER)
            .error(ERROR)
            .valid(VALID)
            .invalid(INVALID)
    };

    let cmd = clap::Command::new("cargo")
        .author("Felix Prasanna")
        .about("format code into perfect rectangles")
        .bin_name("cargo")
        .subcommand_required(true)
        .subcommand(
            clap::command!("unfmt")
                .arg(
                    clap::arg!(<PATH> "unformat source files in <PATH>")
                        .value_parser(clap::value_parser!(std::path::PathBuf)),
                )
                .arg(
                    Arg::new("width")
                        .short('w')
                        .long("line-width")
                        .help("unformat lines to width")
                        .default_value("80")
                        .value_parser(clap::value_parser!(usize)),
                )
                .arg(
                    Arg::new("ignore")
                        .short('i')
                        .long("ignore")
                        .help("ignore files that match regex")
                        .value_parser(clap::value_parser!(Regex)),
                ),
        )
        .styles(styles);

    let matches = cmd.get_matches();
    let matches = match matches.subcommand() {
        Some(("unfmt", matches)) => matches,
        _ => unreachable!("clap L"),
    };

    let search_path = matches
        .get_one::<PathBuf>("PATH")
        .expect("clap handles this");
    let width = matches.get_one::<usize>("width").expect("default is 80");
    let re = matches.get_one::<Regex>("ignore");

    for file in WalkDir::new(search_path) {
        let file = file.context("failed to walkdir file")?;

        if file.file_type().is_dir() {
            continue;
        }

        let path = file.path();

        if re.is_some_and(|re| re.find(path.as_os_str().as_bytes()).is_some()) {
            continue;
        }

        if path.extension().is_some_and(|ext| ext == "rs") {
            let src = fs::read_to_string(path)
                .with_context(|| format!("failed to read source file: {path:?}"))?;

            if src.starts_with('\u{feff}') {
                match cargo_unfmt::unformat(&src['\u{feff}'.len_utf8()..], *width) {
                    Ok(unformatted) => {
                        let mut unformatted_with_bom = String::from("\u{feff}").into_bytes();
                        unformatted_with_bom.extend(unformatted);
                        fs::write(path, &unformatted_with_bom)
                            .context("failed to write formatted source over")?
                    }
                    Err(e) => eprintln!("[cargo-unfmt] error: {e}"),
                }
            } else {
                match cargo_unfmt::unformat(&src, *width) {
                    Ok(unformatted) => fs::write(path, &unformatted)
                        .context("failed to write formatted source over")?,
                    Err(e) => eprintln!("[cargo-unfmt] error: {e}"),
                }
            }
        }
    }

    Ok(())
}
