use std::borrow::Borrow;

use anyhow::Result;
use pulldown_cmark::Event;
use pulldown_cmark_to_cmark::{cmark_resume_with_options, Options};

/// Render Markdown events as Markdown.
pub fn to_cmark<'a, I, E>(events: I) -> Result<String>
where
    I: IntoIterator<Item = E>,
    E: Borrow<Event<'a>>,
{
    let mut buf = String::new();
    let opts = Options {
        code_block_token_count: 3,
        list_token: '-',
        ..Default::default()
    };
    cmark_resume_with_options(events.into_iter(), &mut buf, None, opts)?.finalize(&mut buf)?;
    Ok(buf)
}
