use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Portion
{
    pub source_id: Uuid,
    pub component_id: Uuid,
    pub amount: f64,
}

impl Portion
{
    pub fn of(
        source_id: Uuid,
        component_id: Uuid,
        amount: f64,
    ) -> Self
    {
        Self {
            source_id,
            component_id,
            amount,
        }
    }
}
