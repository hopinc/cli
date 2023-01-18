use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct PaymentMethod {
    pub id: String,
    pub brand: String,
    pub exp_month: u8,
    pub exp_year: u16,
    pub last4: u16,
    pub default: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PaymentMethods {
    pub payment_methods: Vec<PaymentMethod>,
}
