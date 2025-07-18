use pulldown_cmark::{CowStr, Event, LinkType, Tag, TagEnd};
use regex_macro::regex;

use crate::Links;
use crate::config::Config;

/// Fixes intra-doc links.
pub fn fix<'a>(config: &Config, links: &mut Links, events: Vec<Event<'a>>) -> Vec<Event<'a>> {
    let mut iter = events.into_iter().peekable();
    let mut events = Vec::new();

    while let Some(event) = iter.next() {
        match event {
            Event::Start(Tag::Link { dest_url, .. })
                if !regex!(r"^(#|(?:[a-z+]+:)?//)").is_match(&dest_url) =>
            {
                let mut link_text = Vec::new();
                loop {
                    match iter.next().unwrap() {
                        Event::End(TagEnd::Link) => break,
                        e => link_text.push(e),
                    }
                }

                match config.links.get(&*dest_url) {
                    Some(link_dest) => {
                        replace_dest(
                            &mut events,
                            links,
                            link_text,
                            dest_url.to_string(),
                            link_dest.to_string(),
                        );
                    }
                    None => {
                        // See if the link text matches
                        match &*link_text {
                            &[Event::Code(CowStr::Borrowed(text))] => {
                                match config.links.get(text).cloned() {
                                    Some(link_dest) => {
                                        replace_dest(
                                            &mut events,
                                            links,
                                            link_text,
                                            text.to_owned(),
                                            link_dest,
                                        );
                                    }
                                    None => events.extend(link_text),
                                }
                            }
                            _ => {
                                eprintln!("warn: unprocessed link `{dest_url}`");
                                events.extend(link_text);
                            }
                        }
                    }
                }
            }

            Event::Text(text) if text.as_ref() == "[" => {
                let mut maybe_link_text = Vec::new();
                loop {
                    match iter.next().unwrap() {
                        Event::Text(text) if text.as_ref() == "]" => break,
                        e => maybe_link_text.push(e),
                    }
                }

                match &*maybe_link_text {
                    &[Event::Code(CowStr::Borrowed(text))] => {
                        match config.links.get(text).cloned() {
                            Some(link_dest) => {
                                replace_dest(
                                    &mut events,
                                    links,
                                    maybe_link_text,
                                    link_ref(text),
                                    link_dest,
                                );

                                // Skip over what looks like was the link destination.
                                if matches!(iter.peek(), Some(Event::Text(text)) if text.as_ref() == "[")
                                {
                                    iter.next().unwrap(); // [
                                    loop {
                                        match iter.next().unwrap() {
                                            Event::Text(text) if text.as_ref() == "]" => break,
                                            _ => {}
                                        }
                                    }
                                }
                            }
                            None => {
                                let mut maybe_link_ref = Vec::new();
                                // Extract what looks like the link ref
                                if matches!(iter.peek(), Some(Event::Text(text)) if text.as_ref() == "[")
                                {
                                    iter.next().unwrap();
                                    loop {
                                        match iter.next().unwrap() {
                                            Event::Text(text) if text.as_ref() == "]" => break,
                                            e => maybe_link_ref.push(e),
                                        }
                                    }
                                }

                                match &*maybe_link_ref {
                                    &[Event::Text(CowStr::Borrowed(link_ref))] => {
                                        match config.links.get(link_ref).cloned() {
                                            Some(link_dest) => replace_dest(
                                                &mut events,
                                                links,
                                                maybe_link_text,
                                                link_ref.to_owned(),
                                                link_dest,
                                            ),
                                            None => {
                                                println!("warn: unprocessed link `{text}`");
                                                events.extend(maybe_link_text);
                                            }
                                        }
                                    }
                                    _ => {
                                        events.extend(maybe_link_text);
                                        events.extend(maybe_link_ref)
                                    }
                                }
                            }
                        }
                    }
                    _ => events.extend(maybe_link_text),
                }
            }
            event => events.push(event),
        }
    }

    events
}

fn replace_dest<'a>(
    events: &mut Vec<Event<'a>>,
    links: &mut Links,
    link_text: Vec<Event<'a>>,
    link_ref: String,
    link_dest: String,
) {
    let links = links.entry(link_ref.clone()).or_default();
    let i = match links.iter().position(|u| *u == link_dest) {
        Some(i) => i,
        None => {
            let i = links.len();
            links.push(link_dest.clone());
            i
        }
    };
    let id = if i == 0 {
        link_ref
    } else {
        format!("{link_ref}-{i}")
    };

    events.push(Event::Start(Tag::Link {
        link_type: LinkType::Reference,
        dest_url: CowStr::Boxed(link_dest.into_boxed_str()),
        title: CowStr::Borrowed(""),
        id: CowStr::Boxed(id.into_boxed_str()),
    }));
    events.extend(link_text);
    events.push(Event::End(TagEnd::Link));
}

/// Converts a link text to a link reference.
fn link_ref(text: &str) -> String {
    let text = match text.find('<') {
        Some(i) => &text[..i],
        None => text,
    };
    regex!(r"[^\w\-: ]")
        .replace_all(&text.replace(' ', "-"), "")
        .into_owned()
}
