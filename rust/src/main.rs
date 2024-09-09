use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .expect("Please provide a path to a markdown file");
    let path = std::path::Path::new(&path);

    let file = fs::read_to_string(path)?;

    let start = std::time::Instant::now();

    for _ in 0..1000 {
        // let node = markdown::to_mdast(&file, &markdown::ParseOptions::default());

        let parser = pulldown_cmark::Parser::new(&file);

        let mut html_output = String::new();
        pulldown_cmark::html::push_html(&mut html_output, parser);

        println!("{}", html_output.len());
    }

    println!("Time: {:?}", start.elapsed());

    Ok(())
}
