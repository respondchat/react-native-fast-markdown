fn main() -> Result<(), markdown::message::Message> {
    println!(
        "{:?}",
        markdown::to_mdast("# Hey, *you*!", &markdown::ParseOptions::default())?
    );

    Ok(())
}
