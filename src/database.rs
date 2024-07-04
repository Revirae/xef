// #![allow(unused)]
use std::fs::File;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::model::{inventory::Inventory, item::Item, portion::Portion};

fn save_to_file<T: Serialize>(filename: &str, data: &T) -> Result<()>
{
    let file = File::create(filename)?;
    serde_json::to_writer(file, data)?;
    Ok(())
}

fn load_from_file<T: for<'de> Deserialize<'de>>(
    filename: &str,
) -> Result<T>
{
    let file = File::open(filename)?;
    let data = serde_json::from_reader(file)?;
    Ok(data)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AppData
{
    pub items: Vec<Item>,
    pub portions: Vec<Portion>,
}

impl AppData
{
    pub fn save(&self, filename: &str) -> Result<()>
    {
        save_to_file(filename, self)
    }
    pub fn load(filename: &str) -> Result<Self>
    {
        load_from_file(filename)
    }
}

impl From<Inventory> for AppData
{
    fn from(inventory: Inventory) -> Self
    {
        let items: Vec<Item> =
            inventory.list_item().into_iter().map(|c: Item| c).collect();

        let portions = inventory.get_all_portions();
        AppData { items, portions }
    }
}
