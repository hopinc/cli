use std::io::Write;

use anyhow::Result;
use hop::webhooks::types::{PossibleEvents, Webhook, EVENT_CATEGORIES, EVENT_NAMES};
use tabwriter::TabWriter;

pub fn format_webhooks(webhooks: &[Webhook], title: bool) -> Vec<String> {
    let mut tw = TabWriter::new(vec![]);

    if title {
        writeln!(&mut tw, "ID\tURL\tACTIVE EVENTS").unwrap();
    }

    for webhook in webhooks {
        writeln!(
            &mut tw,
            "{}\t{}\t{}",
            webhook.id,
            webhook.webhook_url,
            webhook.events.len()
        )
        .unwrap();
    }

    String::from_utf8(tw.into_inner().unwrap())
        .unwrap()
        .lines()
        .map(std::string::ToString::to_string)
        .collect()
}

pub fn string_to_event(string: &str) -> Result<PossibleEvents> {
    serde_json::from_str(string).map_err(|e| e.into())
}

pub fn get_formatted_events() -> Result<Vec<String>> {
    let mut events = vec![];

    let mut start_idx = 0usize;

    for (name, end_idx) in EVENT_CATEGORIES {
        let end_idx = end_idx as usize + start_idx;

        for (_, event) in &EVENT_NAMES[start_idx..end_idx] {
            events.push(format!("{name}: {event}"));
        }

        start_idx = end_idx;
    }

    Ok(events)
}
