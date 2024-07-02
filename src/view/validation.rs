use anyhow::{anyhow, Result};
use uom::si::f64::Mass;

use crate::parse_mass_amount;

pub fn name_validation(text: String) -> Result<String>
{
    if text.is_empty() {
        Err(anyhow!("empty name"))
    } else {
        Ok(text)
    }
}
pub fn amount_validation(text: String) -> Result<Mass>
{
    let maybe_mass = parse_mass_amount(text.clone());
    maybe_mass.ok_or(anyhow!(
        "{} could not be parsed to Mass",
        text
    ))
}
pub fn price_validation(text: String) -> Result<f64>
{
    let n = text.parse()?;
    if n <= 0.0 {
        Err(anyhow!("{} is a bad price", n))
    } else {
        Ok(n)
    }
}
