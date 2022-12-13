use pulldown_cmark::{Event, Tag};

/// Extract the summary and the rest of the events.
pub fn fix(events: Vec<Event>) -> (Vec<Event>, Vec<Event>) {
    let mut iter = events.into_iter();
    let mut left = Vec::new();
    let mut right = Vec::new();
    let mut count = 0;
    for event in iter.by_ref() {
        match event {
            Event::Start(Tag::Paragraph) => {
                count += 1;
                left.push(event);
            }
            Event::End(Tag::Paragraph) => {
                count -= 1;
                left.push(event);
                if count == 0 {
                    break;
                }
            }
            event => left.push(event),
        }
    }
    right.extend(iter);
    (left, right)
}
