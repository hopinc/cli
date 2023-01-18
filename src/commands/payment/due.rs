use std::collections::HashMap;

use anyhow::Result;
use clap::Parser;

use super::utils::{
    format_payment_methods, get_all_payment_methods, get_all_projects_for_payment_method,
};
use crate::commands::projects::finance::utils::get_project_balance;
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "Check how much is due for a payment method(s)")]
pub struct Options {
    #[clap(
        help = "The ID(s) of the payment method(s), if not provided all payment methods will be used"
    )]
    pub payment_methods: Vec<String>,
}

pub async fn handle(options: &Options, state: &State) -> Result<()> {
    let payment_methods = get_all_payment_methods(&state.http).await?;

    let payment_methods = if options.payment_methods.is_empty() {
        payment_methods
    } else {
        payment_methods
            .into_iter()
            .filter(|d| options.payment_methods.contains(&d.id))
            .collect()
    };

    let mut payment_method_balance = HashMap::new();

    for payment_method in &payment_methods {
        let projects = get_all_projects_for_payment_method(&state.http, &payment_method.id).await?;

        let mut balances = Vec::new();

        for project in projects.into_iter() {
            let balance = get_project_balance(&state.http, &project.id).await?;

            balances.push((balance, project));
        }

        payment_method_balance.insert(payment_method.id.clone(), balances);
    }

    let payment_methods_fmt = format_payment_methods(&payment_methods, false)?;

    let now = chrono::Local::now().date_naive();

    for (payment_method_fmt, payment_method) in payment_methods_fmt
        .into_iter()
        .zip(payment_methods.into_iter())
    {
        println!("{}", payment_method_fmt);

        let balances = payment_method_balance.get(&payment_method.id).unwrap();

        if balances.is_empty() {
            println!("  No projects found");
        }

        for (balance, project) in balances.iter() {
            let next_billing_date =
                chrono::NaiveDate::parse_from_str(&balance.next_billing_cycle, "%Y-%m-%d")? - now;

            print!(" `{}`, ", project.name);

            print!(
                "${:.2} due in {} days",
                balance.balance.parse::<f64>()? - balance.outstanding.parse::<f64>()?,
                next_billing_date.num_days()
            );

            if balance.outstanding != "0.00" {
                print!(" + ${} outstanding", balance.outstanding);
            }

            println!();
        }
    }

    Ok(())
}
