use anyhow::Result;
use std::io::Write;
use tabwriter::TabWriter;

use crate::{state::http::HttpClient, utils::capitalize};

use super::types::{PaymentMethod, PaymentMethods};

pub async fn get_all_payment_methods(http: &HttpClient) -> Result<Vec<PaymentMethod>> {
    let data = http
        .request::<PaymentMethods>("GET", "/billing/@me/payment-methods", None)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Error while parsing response"))?
        .payment_methods;

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
