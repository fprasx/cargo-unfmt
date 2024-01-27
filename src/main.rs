use std::{fs, path::Path};

use anyhow::Context;
use cargo_unfmt::{formatters::BlockUnformatter, morpheme::Tokens, Unformat};
use walkdir::WalkDir;

fn main() -> anyhow::Result<()> {
    unfmt()
}

fn unfmt() -> anyhow::Result<()> {
    let input = Path::new("/Users/fpx/code/rust/cargo-unfmt/test_crates/input/");
    let output = Path::new("/Users/fpx/code/rust/cargo-unfmt/test_crates/output/");
    fs::create_dir_all(output).context("failed to create output directory")?;

    for file in WalkDir::new(input) {
        let file = file.context("failed to walkdir file")?;
        if file.file_type().is_dir() || file.path().to_str().unwrap().contains(".git/") {
            continue;
        }

        let relative = file.path().strip_prefix(input).unwrap();
        let path = output.join(relative);
        fs::create_dir_all(path.parent().unwrap())
            .context("failed to create output subdirectory")?;
        fs::File::create(&path).context("failed to create new file")?;

        if path.extension().is_some_and(|ext| ext == "rs") {
            let src =
                String::from_utf8(fs::read(file.path()).context("failed to read source file")?)
                    .context("file was not utf-8")?;
            let formatted = BlockUnformatter::<80>.unformat(Tokens::tokenize(&src).tokens());
            fs::write(&path, &formatted).context("failed to write formatted source over")?;
        } else {
            fs::copy(file.path(), &path).context("failed to copy file over")?;
        }
    }
    Ok(())
}
