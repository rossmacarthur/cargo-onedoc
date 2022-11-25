use anyhow::{anyhow, Result};
use pulldown_cmark::{CodeBlockKind, CowStr, Event, Tag};

/// Fixes code blocks.
pub fn fix(events: Vec<Event>) -> Result<Vec<Event>> {
    let mut iter = events.into_iter();
    let mut events = Vec::new();
    while let Some(event) = iter.next() {
        match event {
            Event::Start(Tag::CodeBlock(kind)) if is_rust(&kind) => {
                let tag = Tag::CodeBlock(fix_code_block_kind(kind));
                events.push(Event::Start(tag.clone()));

                loop {
                    match iter.next().unwrap() {
                        Event::Text(code) => {
                            events.push(Event::Text(fix_code_block(code)));
                        }
                        Event::End(Tag::CodeBlock(_)) => {
                            events.push(Event::End(tag));
                            break;
                        }
                        event => {
                            return Err(anyhow!(
                                "expected Event::End(..), got event `{:?}`",
                                event
                            ));
                        }
                    }
                }
            }
            event => events.push(event),
        }
    }
    Ok(events)
}

/// Returns true if a code block is a Rust one.
fn is_rust(kind: &CodeBlockKind) -> bool {
    match kind {
        CodeBlockKind::Fenced(attr) if attr.is_empty() => true,
        CodeBlockKind::Fenced(attr) if attr.as_ref() == "rust" => true,
        CodeBlockKind::Fenced(_) => false,
        _ => true,
    }
}

/// Makes empty code blocks `rust` code blocks.
fn fix_code_block_kind(kind: CodeBlockKind) -> CodeBlockKind {
    match kind {
        CodeBlockKind::Fenced(attr) if attr.is_empty() => {
            CodeBlockKind::Fenced(CowStr::Borrowed("rust"))
        }
        kind => kind,
    }
}

/// Rewrites code blocks to exclude `#` prefixed lines.
fn fix_code_block(code: CowStr) -> CowStr {
    let mut result = String::new();
    for line in code
        .lines()
        .filter(|line| !line.trim().starts_with("# ") && *line != "#")
    {
        result.push_str(line);
        result.push('\n');
    }
    CowStr::Boxed(result.into_boxed_str())
}
