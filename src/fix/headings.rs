use pulldown_cmark::{Event, Tag, TagEnd};

/// Increases each heading level by one.
pub fn fix(events: Vec<Event>) -> Vec<Event> {
    let mut iter = events.into_iter();
    let mut events = Vec::new();
    while let Some(event) = iter.next() {
        match event {
            Event::Start(Tag::Heading {
                level,
                id,
                classes,
                attrs,
            }) => {
                let level = (level as usize + 1).try_into().unwrap();
                let tag = Tag::Heading {
                    level,
                    id,
                    classes,
                    attrs,
                };
                let tag_end = TagEnd::Heading(level);
                events.push(Event::Start(tag));
                loop {
                    match iter.next().unwrap() {
                        Event::End(TagEnd::Heading(..)) => break,
                        event => events.push(event),
                    }
                }
                events.push(Event::End(tag_end));
            }
            event => events.push(event),
        }
    }
    events
}
