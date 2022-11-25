use pulldown_cmark::{Event, Tag};

/// Increases each heading level by one.
pub fn fix(events: Vec<Event>) -> Vec<Event> {
    let mut iter = events.into_iter();
    let mut events = Vec::new();
    while let Some(event) = iter.next() {
        match event {
            Event::Start(Tag::Heading(level, frag, classes)) => {
                let tag = Tag::Heading((level as usize + 1).try_into().unwrap(), frag, classes);
                events.push(Event::Start(tag.clone()));
                loop {
                    match iter.next().unwrap() {
                        Event::End(Tag::Heading(..)) => break,
                        event => events.push(event),
                    }
                }
                events.push(Event::End(tag));
            }
            event => events.push(event),
        }
    }
    events
}
