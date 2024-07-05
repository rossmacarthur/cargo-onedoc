use pulldown_cmark::{Event, LinkType, Tag, TagEnd};
use regex_macro::regex;

use crate::Context;

/// Fixes relative file links.
pub fn fix<'a>(ctx: &Context, events: Vec<Event<'a>>) -> Vec<Event<'a>> {
    let mut iter = events.into_iter();
    let mut events = Vec::new();
    while let Some(event) = iter.next() {
        match event {
            Event::Start(Tag::Link {
                link_type: LinkType::Inline,
                dest_url,
                title,
                id,
            }) if !regex!(r"^(#|(?:[a-z+]+:)?//)").is_match(&dest_url) => {
                let i = dest_url.find('#').unwrap_or(dest_url.len());
                let dst_url_no_frag: &str = &dest_url[..i];
                let fragment: &str = &dest_url[i..];
                match ctx.config.links.get(dst_url_no_frag).cloned() {
                    Some(new_dst) => {
                        events.push(Event::Start(Tag::Link {
                            link_type: LinkType::Inline,
                            dest_url: format!("{new_dst}{fragment}").into(),
                            title: title.clone(),
                            id,
                        }));
                        loop {
                            match iter.next().unwrap() {
                                Event::End(TagEnd::Link) => break,
                                event => events.push(event),
                            }
                        }
                        events.push(Event::End(TagEnd::Link));
                    }
                    None => {
                        eprintln!("warn: unprocessed link `{}`", dst_url_no_frag);
                        events.push(iter.next().unwrap());
                    }
                }
            }
            event => events.push(event),
        }
    }
    events
}
