use std::fs;

use pulldown_cmark::{Event, Parser};
use serde_json::json;

#[cfg(test)]
mod tests {
    use crate::{parse_markdown, MarkdownOptions};

    use super::*;

    #[test]
    fn test_markdown_parsing() -> Result<(), Box<dyn std::error::Error>> {
        let path = std::path::Path::new("../TEST.md");

        let file = fs::read_to_string(path)?;

        let result = parse_markdown(&file, &MarkdownOptions::default());
        let duration = std::time::Instant::now();

        println!("{:?}", result);

        println!("{:?}", duration.elapsed());

        Ok(())
    }
}
