use std::{fs, path::Path};

use anyhow::Context;
use cargo_unfmt::{formatters::BlockUnformatter, morpheme::Tokenizer, Unformat};
use walkdir::WalkDir;

fn main() -> anyhow::Result<()> {
    test_rustfmt()
}

fn test_rustfmt() -> anyhow::Result<()> {
    let input = Path::new("/Users/fpx/code/rust/cargo-unfmt/test_crates/input/");
    let output = Path::new("/Users/fpx/code/rust/cargo-unfmt/test_crates/output/");
    fs::create_dir_all(output).context("failed to create output directory")?;

    for file in WalkDir::new(input) {
        let file = file.context("failed to walkdir file")?;
        let path_str = file.path().to_str().unwrap();
        if file.file_type().is_dir() || path_str.contains(".git") || path_str.contains("tests")
            {
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
            let formatted = BlockUnformatter::<80>.unformat(
                &Tokenizer::tokenize_file(&src)
                    .with_context(|| format!("failed to parse: {:?}", file.path()))?,
            );
            fs::write(&path, &formatted).context("failed to write formatted source over")?;
        } else {
            fs::copy(file.path(), &path).context("failed to copy file over")?;
        }
    }
    Ok(())
}
