use anyhow::Context;

fn main() -> anyhow::Result<()> {
    let src = include_str!("../long-rust-file.rs");

    let parsed = syn::parse_file(src).context("failed to parse file")?;

    // for item in parsed.items {
    //     println!("{item:?}")
    // }

    Ok(())
}
