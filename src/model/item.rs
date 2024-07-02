use anyhow::Result;
use serde::{Deserialize, Serialize};
use uom::si::f64::Mass;
use uuid::Uuid;

#[derive(Default)]
pub struct ItemBuilder
{
    id: Option<Uuid>,
    name: Option<String>,
    amount: Option<Mass>,
    price: Option<f64>,
}

impl ItemBuilder
{
    pub fn build(&self) -> Result<Item>
    {
        let id = self.id.unwrap_or(Uuid::new_v4());
        let name = self.name.clone().unwrap_or_default();
        let amount = self.amount.unwrap_or_default();
        let price = self.price.unwrap_or_default();
        let item = Item {
            id,
            name,
            amount,
            price,
        };
        Ok(item)
    }
    pub fn with_id(&mut self, new_id: Uuid) -> &mut Self
    {
        self.id = Some(new_id);
        self
    }
    pub fn with_name(&mut self, new_name: &str) -> &mut Self
    {
        self.name = Some(String::from(new_name));
        self
    }
    pub fn with_amount(
        &mut self,
        new_amount: Mass,
    ) -> &mut Self
    {
        self.amount = Some(new_amount);
        self
    }
    pub fn with_price(&mut self, new_price: f64) -> &mut Self
    {
        self.price = Some(new_price);
        self
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Item
{
    pub id: Uuid,
    pub name: String,
    pub amount: Mass,
    pub price: f64,
}

impl Item
{
    pub fn builder() -> ItemBuilder { ItemBuilder::default() }
    pub fn new(name: &str) -> Self
    {
        Self::builder().with_name(name).build().unwrap()
    }
}
