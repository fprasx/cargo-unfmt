use cargo_unfmt::emit;
use syn::visit::Visit;

fn main() -> anyhow::Result<()> {
    let src = include_str!("../test_files/short-rust-file.rs");

    let mut vis = cargo_unfmt::location::Visitor::new();
    vis.visit_file(&syn::parse_file(src).unwrap());
    for region in vis.regions() {
        println!("{region:?}")
    }

    let uf = cargo_unfmt::unformat(src).unwrap();

    let mut bytes = vec![];
    emit::block(&mut bytes, uf.tokens().to_vec(), 80);

    println!("{}", String::from_utf8(bytes).unwrap());
    Ok(())
}
