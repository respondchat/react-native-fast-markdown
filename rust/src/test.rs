use std::fs;

use pulldown_cmark::{Event, Parser};
use serde_json::json;

#[cfg(test)]
mod tests {
    use linkify::LinkFinder;

    use crate::{parse_markdown, MarkdownOptions, LINKIFY};

    use super::*;

    #[test]
    fn test_markdown_parsing() -> Result<(), Box<dyn std::error::Error>> {
        let path = std::path::Path::new("../TEST.md");

        let file = fs::read_to_string(path)?;

        let mut linkify = LinkFinder::new();
        linkify.url_can_be_iri(false);
        linkify.url_must_have_scheme(true);
        linkify.kinds(&[linkify::LinkKind::Url]);
        unsafe {
            LINKIFY.as_mut_ptr().write(linkify);
        };

        let mut width = 0;

        for event in Parser::new(&file) {
            if let Event::End(_) = event {
                width -= 2;
            }
            eprintln!("  {:width$}{event:?}", "");
            if let Event::Start(_) = event {
                width += 2;
            }
        }

        let duration = std::time::Instant::now();

        let result = parse_markdown(&file, &MarkdownOptions::default());

        println!("{:?}", result);

        println!("{:?}", duration.elapsed());

        Ok(())
    }
}
