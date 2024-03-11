use cargo_unfmt::{
    formatters::BlockUnformatter,
    lex::{self, Spanned},
    Unformat,
};

use std::{fs, path::Path};

use anyhow::Context;
use walkdir::WalkDir;

pub fn test_crate(krate: &str) -> anyhow::Result<()> {
    let input_dir = format!("/Users/fpx/code/rust/cargo-unfmt/test_crates/input/{krate}");
    let input_dir = Path::new(&input_dir);
    let output_dir = format!("/Users/fpx/code/rust/cargo-unfmt/test_crates/output/{krate}");
    let output_dir = Path::new(&output_dir);

    fs::create_dir_all(output_dir).context("failed to create output directory")?;

    for file in WalkDir::new(input_dir) {
        let file = file.context("failed to walkdir file")?;

        // If you figure out how to format a directory let me know
        if file.file_type().is_dir() {
            continue;
        }

        // Ignore tests directories, since those often contain code that needs
        // to be formatted a certain way, a least for rustfmt, and probably for
        // rustc
        if file
            .path()
            .to_str()
            .map(|file| file.contains("tests"))
            .unwrap_or(false)
        {
            continue;
        }

        // Trim out just the ~/.../test_crates/input/ - we still want the rest of
        // the directories to preserve the project structure
        let src_path = file.path().strip_prefix(input_dir).unwrap();
        let path = output_dir.join(src_path);

        // Create subdirectory if need be, ex, attempting to create src/foo/bar/baz.rs
        // out of nothing will return directory src/foo/bar does not exist
        let parent = path.parent().unwrap();
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create output subdirectory: {parent:?}"))?;
        fs::File::create(&path).with_context(|| format!("failed to create new file {path:?}"))?;

        if path.extension().is_some_and(|ext| ext == "rs") {
            let src = fs::read_to_string(file.path())
                .with_context(|| format!("failed to read source file: {path:?}"))?;
            let formatted = BlockUnformatter::<80>.unformat(
                &lex::lex_file(&src)
                    .with_context(|| format!("failed to parse: {:?}", file.path()))?
                    .into_iter()
                    .map(Spanned::into_inner)
                    .collect::<Vec<_>>(),
            );
            fs::write(&path, &formatted).context("failed to write formatted source over")?;
        } else {
            fs::copy(file.path(), &path).context("failed to copy file over")?;
        }
    }

    Ok(())
}

#[test]
fn rustfmt() {
    test_crate("rustfmt").expect("failed to unformat rustfmt");
}
