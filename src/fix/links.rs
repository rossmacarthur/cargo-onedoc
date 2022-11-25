use pulldown_cmark::{CowStr, Event, LinkType, Tag};

/// Fixes intra-doc links.
pub fn fix(events: Vec<Event>) -> Vec<Event> {
    let mut iter = events.into_iter().peekable();
    let mut events = Vec::new();
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
                    &[Event::Code(CowStr::Borrowed(text))] => match maybe_link(text) {
                        Some(uri) => {
                            let tag = Tag::Link(
                                LinkType::Inline,
                                CowStr::Borrowed(uri),
                                CowStr::Borrowed(""),
                            );
                            events.push(Event::Start(tag.clone()));
                            events.push(Event::Code(CowStr::Borrowed(text)));
                            events.push(Event::End(tag.clone()));
                        }
                        None => events.extend(local),
                    },
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
    events
}

fn maybe_link(_text: &str) -> Option<&'static str> {
    None
}
