#![feature(trait_alias)]
#![feature(downcast_unchecked)]

use anyhow::Result;
use database::AppData;
use model::inventory::Inventory;
use uom::{
    fmt::DisplayStyle,
    si::{
        f64::Mass,
        mass::{self, gram, kilogram},
    },
};
use uuid::Uuid;
pub mod database;
pub mod model;
pub mod view;

#[derive(Clone, Default, Debug, PartialEq)]
pub enum AppMode
{
    #[default]
    InsertMode,
    EditMode(Uuid),
    PortionMode(Uuid, Uuid),
}

#[derive(Clone, Default)]
pub struct AppState
{
    pub model: Inventory,
    pub mode: AppMode,
}

impl AppState
{
    pub fn set_data(&mut self, data: AppData)
    -> Result<()>
    {
        self.model = data.try_into()?;
        Ok(())
    }
}

//out
pub fn mass_format<U>(
    unit: U,
    value: f64,
    style: DisplayStyle,
) -> String
where
    U: mass::Unit + uom::Conversion<f64, T = f64>,
{
    Mass::new::<kilogram>(value)
        .into_format_args(unit, style)
        .to_string()
}
pub fn mass_format_logic1(
    value: f64,
    style: DisplayStyle,
) -> String
{
    if value > 0.999 {
        mass_format(kilogram, value, style)
    } else {
        mass_format(gram, value, style)
    }
}
pub fn clip_uuid(id: Uuid, index: usize) -> String
{
    let id = id.to_string().into_boxed_str();
    let (id, _) = id.split_at(index);
    id.to_string()
}

//in
pub fn parse_mass_amount(text: String) -> Option<Mass>
{
    let index = text.find(' ')?;
    let (value, unit) = text.split_at(index);
    let amount: f64 = value.parse().ok()?;
    match unit.trim_start() {
        "g" | "grama" | "gramas" => {
            Some(Mass::new::<gram>(amount))
        }
        "kg" | "kilo" | "kilos" => {
            Some(Mass::new::<kilogram>(amount))
        }
        _ => None,
    }
}
