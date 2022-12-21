use pulldown_cmark::{Event, LinkType, Tag};
use regex_macro::regex;

use crate::Context;

/// Fixes relative file links.
pub fn fix<'a>(ctx: &Context, events: Vec<Event<'a>>) -> Vec<Event<'a>> {
    let mut iter = events.into_iter();
    let mut events = Vec::new();
    while let Some(event) = iter.next() {
        match event {
            Event::Start(Tag::Link(LinkType::Inline, dst, title))
                if !regex!(r"^(#|(?:[a-z+]+:)?//)").is_match(&dst) =>
            {
                let i = dst.find('#').unwrap_or(dst.len());
                let dst_no_frag: &str = &dst[..i];
                let fragment: &str = &dst[i..];
                match ctx.config.links.get(dst_no_frag).cloned() {
                    Some(new_dst) => {
                        events.push(Event::Start(Tag::Link(
                            LinkType::Inline,
                            format!("{new_dst}{fragment}").into(),
                            title.clone(),
                        )));
                        loop {
                            match iter.next().unwrap() {
                                Event::End(Tag::Link(LinkType::Inline, _, _)) => break,
                                event => events.push(event),
                            }
                        }
                        events.push(Event::End(Tag::Link(
                            LinkType::Reference,
                            format!("{new_dst}{fragment}").into(),
                            title,
                        )));
                    }
                    None => {
                        eprintln!("warn: unprocessed link `{}`", dst_no_frag);
                        events.push(iter.next().unwrap());
                    }
                }
            }
            event => events.push(event),
        }
    }
    events
}
