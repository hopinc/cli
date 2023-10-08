use std::io::Write;

use anyhow::Result;
use hop::webhooks::types::{PossibleEvents, Webhook};
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
