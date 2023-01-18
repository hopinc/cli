use std::io::Write;

use anyhow::Result;
use tabwriter::TabWriter;

use super::types::{PaymentMethod, PaymentMethods};
use crate::commands::projects::types::Project;
use crate::state::http::HttpClient;
use crate::utils::capitalize;

pub async fn get_all_payment_methods(http: &HttpClient) -> Result<Vec<PaymentMethod>> {
    let data = http
        .request::<PaymentMethods>("GET", "/billing/@me/payment-methods", None)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Error while parsing response"))?
        .payment_methods;

    Ok(data)
}

pub async fn get_all_projects_for_payment_method(
    http: &HttpClient,
    payment_method_id: &str,
) -> Result<Vec<Project>> {
    let data = http
        .request::<Vec<Project>>(
            "GET",
            &format!("/billing/payment-methods/{payment_method_id}/projects"),
            None,
        )
        .await?
        .ok_or_else(|| anyhow::anyhow!("Error while parsing response"))?;

    Ok(data)
}

pub fn format_payment_methods(
    payment_methods: &[PaymentMethod],
    title: bool,
) -> Result<Vec<String>> {
    let mut tw = TabWriter::new(vec![]);

    if title {
        writeln!(&mut tw, "BRAND\tNUMBER\tEXPIRATION")?;
    }

    for payment_method in payment_methods {
        let stars = match payment_method.brand.as_str() {
            // amex is extra 🙄
            "amex" => "**** ****** *",
            _ => "**** **** **** ",
        };

        writeln!(
            &mut tw,
            "{}\t{}{:04}\t{:02}/{}",
            capitalize(&payment_method.brand),
            stars,
            payment_method.last4,
            payment_method.exp_month,
            payment_method.exp_year,
        )?;
    }

    let out = String::from_utf8(tw.into_inner().unwrap())?
        .lines()
        .map(std::string::ToString::to_string)
        .collect();

    Ok(out)
}
