//! # wcd
//!
//! Parser for the `.wcd` (web clean document) format.
//!
//! `.wcd` is the PoC's frozen, declarative document format. It is line-oriented
//! and deliberately tiny: a version line, then headings, paragraphs, list
//! items, and links. There are no inline styles, no nesting, and - crucially -
//! no executable content. A document is pure data; rendering it requires no
//! code from the document itself. That is the entire point of document mode.
//!
//! ## Grammar
//!
//! ```text
//! # wcd/1                          first non-blank line: the version marker
//! h1 Heading text                  level-1 heading
//! h2 Subheading text               level-2 heading
//! p  Paragraph text                paragraph
//! -  List item text                unordered list item
//! link Display text => target.wcd  link to another document
//! # comment                        comment (after the version line); ignored
//! <blank>                          ignored
//! ```
//!
//! This crate is `no_std` so the WASM document renderer can share the exact
//! parser the rest of the system uses.

#![no_std]

extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec::Vec;

/// The version marker that must begin every document.
pub const VERSION_MARKER: &str = "# wcd/1";

/// One parsed document block.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Block {
    /// A heading, with its level (`1` or `2`) and text.
    Heading { level: u8, text: String },
    /// A paragraph of body text.
    Paragraph(String),
    /// An unordered list item.
    ListItem(String),
    /// A link: display text plus the target document path.
    Link { text: String, target: String },
}

/// A parsed document: an ordered sequence of blocks.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Document {
    pub blocks: Vec<Block>,
}

/// Why parsing failed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParseError {
    /// The document did not begin with the `# wcd/1` version marker.
    MissingVersion,
    /// A non-blank line did not match any known block sigil.
    UnknownLine { line_no: usize, content: String },
    /// A `link` line was missing its `=>` separator.
    MalformedLink { line_no: usize },
}

/// Parse `.wcd` source into a [`Document`].
///
/// # Arguments
/// * `source` - the full document text.
///
/// # Returns
/// The parsed [`Document`] on success.
///
/// # Errors
/// Returns [`ParseError`] if the version marker is missing, a line uses an
/// unknown sigil, or a `link` line is malformed.
pub fn parse(source: &str) -> Result<Document, ParseError> {
    let mut lines = source.lines().enumerate();

    // The first non-blank line must be the version marker.
    let mut saw_version = false;
    let mut blocks = Vec::new();

    for (_idx, raw) in lines.by_ref() {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed == VERSION_MARKER {
            saw_version = true;
            break;
        }
        return Err(ParseError::MissingVersion);
    }
    if !saw_version {
        return Err(ParseError::MissingVersion);
    }

    for (idx, raw) in lines {
        let line_no = idx + 1;
        let line = raw.trim_end();
        let trimmed = line.trim_start();

        if trimmed.is_empty() {
            continue;
        }
        // Comments (after the version line) start with '#'.
        if trimmed.starts_with('#') {
            continue;
        }

        let block = parse_block(trimmed, line_no)?;
        blocks.push(block);
    }

    Ok(Document { blocks })
}

/// Parse a single non-blank, non-comment content line.
fn parse_block(line: &str, line_no: usize) -> Result<Block, ParseError> {
    if let Some(rest) = line.strip_prefix("h1 ") {
        return Ok(Block::Heading {
            level: 1,
            text: rest.trim().to_string(),
        });
    }
    if let Some(rest) = line.strip_prefix("h2 ") {
        return Ok(Block::Heading {
            level: 2,
            text: rest.trim().to_string(),
        });
    }
    if let Some(rest) = line.strip_prefix("p ") {
        return Ok(Block::Paragraph(rest.trim().to_string()));
    }
    if let Some(rest) = line.strip_prefix("- ") {
        return Ok(Block::ListItem(rest.trim().to_string()));
    }
    if let Some(rest) = line.strip_prefix("link ") {
        let (text, target) = rest
            .split_once("=>")
            .ok_or(ParseError::MalformedLink { line_no })?;
        return Ok(Block::Link {
            text: text.trim().to_string(),
            target: target.trim().to_string(),
        });
    }

    Err(ParseError::UnknownLine {
        line_no,
        content: line.to_string(),
    })
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn parses_a_full_document() {
        let src = "# wcd/1\n\nh1 Welcome\n# a comment\np Hello there.\n- one\n- two\nlink Next => page-two.wcd\n";
        let doc = parse(src).expect("should parse");
        assert_eq!(
            doc.blocks,
            vec![
                Block::Heading {
                    level: 1,
                    text: "Welcome".to_string()
                },
                Block::Paragraph("Hello there.".to_string()),
                Block::ListItem("one".to_string()),
                Block::ListItem("two".to_string()),
                Block::Link {
                    text: "Next".to_string(),
                    target: "page-two.wcd".to_string()
                },
            ]
        );
    }

    #[test]
    fn requires_version_marker() {
        assert_eq!(parse("h1 No version\n"), Err(ParseError::MissingVersion));
    }

    #[test]
    fn rejects_unknown_sigil() {
        let src = "# wcd/1\nblah nope\n";
        assert!(matches!(parse(src), Err(ParseError::UnknownLine { .. })));
    }

    #[test]
    fn rejects_malformed_link() {
        let src = "# wcd/1\nlink no arrow here\n";
        assert!(matches!(parse(src), Err(ParseError::MalformedLink { .. })));
    }

    #[test]
    fn leading_blank_lines_before_version_are_ok() {
        let src = "\n\n# wcd/1\np body\n";
        assert!(parse(src).is_ok());
    }
}
