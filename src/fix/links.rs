use std::collections::BTreeMap;

use pulldown_cmark::{CowStr, Event, LinkType, Tag};
use regex_macro::regex;

use crate::Context;

pub type Urls = BTreeMap<String, Vec<String>>;

/// Fixes intra-doc links.
pub fn fix<'a>(ctx: &Context, events: Vec<Event<'a>>) -> (Vec<Event<'a>>, Urls) {
    let mut iter = events.into_iter().peekable();
    let mut events = Vec::new();
    let mut urls: BTreeMap<String, Vec<String>> = BTreeMap::new();

    while let Some(event) = iter.next() {
        match event {
            Event::Text(text) if text.as_ref() == "[" => {
                let mut local = Vec::new();
                loop {
                    match iter.next().unwrap() {
                        Event::Text(text) if text.as_ref() == "]" => break,
                        event => local.push(event),
                    }
                }

                match &*local {
                    &[Event::Code(CowStr::Borrowed(text))] => {
                        match ctx.config.links.get(text).cloned() {
                            Some(dest) => {
                                let link_ref = link_ref(text);

                                let urls = urls.entry(link_ref.clone()).or_insert_with(Vec::new);
                                let i = match urls.iter().position(|u| *u == dest) {
                                    Some(i) => i,
                                    None => {
                                        let i = urls.len();
                                        urls.push(dest);
                                        i
                                    }
                                };
                                let actual = if i == 0 {
                                    link_ref
                                } else {
                                    format!("{}-{}", link_ref, i)
                                };

                                let tag = Tag::Link(
                                    LinkType::Reference,
                                    CowStr::Boxed(actual.into_boxed_str()),
                                    CowStr::Borrowed(""),
                                );
                                events.push(Event::Start(tag.clone()));
                                events.push(Event::Code(CowStr::Borrowed(text)));
                                events.push(Event::End(tag.clone()));
                            }
                            None => {
                                eprintln!("warn: unprocessed link `{}`", text);
                                events.extend(local)
                            }
                        }
                    }
                    _ => events.extend(local),
                }

                match iter.peek() {
                    Some(Event::Text(text)) if text.as_ref() == "[" => {
                        iter.next().unwrap();
                        loop {
                            match iter.next().unwrap() {
                                Event::Text(text) if text.as_ref() == "]" => break,
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }
            event => events.push(event),
        }
    }

    (events, urls)
}

fn link_ref(text: &str) -> String {
    let text = match text.find('<') {
        Some(i) => &text[..i],
        None => text,
    };
    regex!(r"[^\w\- ]")
        .replace_all(&text.to_ascii_lowercase().replace(' ', "-"), "")
        .into_owned()
}
