use std::fs;

use pulldown_cmark::{Event, Parser};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .expect("Please provide a path to a markdown file");
    let path = std::path::Path::new(&path);

    let file = fs::read_to_string(path)?;

    let start = std::time::Instant::now();

    eprintln!("[");
	
	for _ in 0..1000 {
		let parser = pulldown_cmark::Parser::new(&file);
        // let node = markdown::to_mdast(&file, &markdown::ParseOptions::default());

        let mut html_output = String::new();
        pulldown_cmark::html::push_html(&mut html_output, parser);

        // println!("{}", html_output);

		// let mut width = 0;
		// for event in Parser::new(&file) {
		// 	if let Event::End(_) = event {
		// 		width -= 2;
		// 	}

			// eprintln!("  {:width$}{event:?}", "");
		// 	if let Event::Start(_) = event {
		// 		width += 2;
		// 	}
		// }

    }

    eprintln!("]");

    println!("Time: {:?}", start.elapsed());

    Ok(())
}
