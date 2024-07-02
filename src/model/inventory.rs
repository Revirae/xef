use std::{cell::RefCell, rc::Rc};

use anyhow::{anyhow, Result};
use indexmap::IndexMap;
use petgraph::{
    algo::is_cyclic_directed,
    graph::{DiGraph, EdgeIndex, NodeIndex},
    visit::EdgeRef,
};
use uuid::Uuid;

use super::{item::Item, portion::Portion};
use crate::database::AppData;

#[derive(Clone, Default)]
pub struct Inventory
{
    graph: DiGraph<Rc<RefCell<Item>>, Portion>,
    nodes: IndexMap<Uuid, NodeIndex>,
}

impl Inventory
{
    pub fn new() -> Self
    {
        Inventory {
            graph: DiGraph::new(),
            nodes: IndexMap::new(),
        }
    }

    pub fn get_node(&self, id: &Uuid) -> Result<&NodeIndex>
    {
        self.nodes.get(id).ok_or(anyhow!("{} not found", id))
    }

    pub fn get_nodes(&self) -> im::Vector<NodeIndex>
    {
        self.nodes.values().copied().collect()
    }

    pub fn get_all_portions(&self) -> Vec<Portion>
    {
        let mut portions = Vec::<Portion>::new();
        for node in self.nodes.values() {
            for edge in self.graph.edges(*node) {
                let portion = edge.weight();
                portions.push(portion.clone());
            }
        }
        portions
    }

    pub fn get_item(&self, id: &Uuid) -> Result<Rc<RefCell<Item>>>
    {
        if let Some(node) = self.nodes.get(id) {
            let item = self.graph[*node].clone();
            Ok(item)
        } else {
            Err(anyhow!(format!("{} not found", id)))
        }
    }

    pub fn remove_component(&mut self, id: &Uuid) -> Result<()>
    {
        let _ = self.nodes.shift_remove(id).ok_or(anyhow!(format!(
            "failed to remove node index for {:?}",
            id
        )))?;
        Ok(())
    }

    pub fn list_components(&self) -> im::Vector<Item>
    {
        self.nodes
            .values()
            .filter_map(|&node| {
                if let Ok(item_ref) = self.graph[node].try_borrow() {
                    Some(item_ref.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn add_item(&mut self, item: Item) -> Result<()>
    {
        let node = Rc::new(RefCell::new(item.clone()));
        let item_id = item.id;
        let node_index = self.graph.add_node(node.clone());
        self.nodes.insert(item_id, node_index);
        Ok(())
    }

    pub fn update_item(&mut self, id: Uuid, item: Item) -> Result<()>
    {
        if let Ok(index) = self.get_node(&id) {
            let item_ref = self.graph[*index].clone();
            let mut item_mref = item_ref.borrow_mut();
            *item_mref = item;
        }
        Ok(())
    }

    pub fn get_portions(&self, id: Uuid) -> Result<im::Vector<Portion>>
    {
        let node = self.get_node(&id)?;
        let edges = self.graph.edges(*node);
        let portions: im::Vector<Portion> =
            edges.into_iter().map(|e| e.weight().clone()).collect();
        // dbg!(portions.clone());
        Ok(portions)
    }

    fn add_portion(&mut self, portion: Portion) -> Result<EdgeIndex>
    {
        // dbg!(portion.clone());
        let source = self.get_node(&portion.source_id)?;
        let component = self.get_node(&portion.component_id)?;
        let index = self.graph.add_edge(*source, *component, portion);
        Ok(index)
    }

    pub fn create_portion(
        &mut self,
        from: Uuid,
        to: Uuid,
        amount: f64,
    ) -> Result<EdgeIndex>
    {
        let source = self.get_node(&to)?;
        let component = self.get_node(&from)?;
        let portion = Portion::of(to, from, amount);
        let index = self.graph.add_edge(*source, *component, portion);
        Ok(index)
    }

    pub fn test_portion(
        &mut self,
        from: Uuid,
        to: Uuid,
        amount: f64,
    ) -> Result<bool>
    {
        let edge = self.create_portion(from, to, amount)?;
        let result = is_cyclic_directed(&self.graph);
        self.graph.remove_edge(edge);
        Ok(result)
    }

    pub fn get_amount_(&self, index: NodeIndex) -> f64
    {
        let mut total_amount = self.graph[index].borrow().amount.value;
        for edge in self.graph.edges(index) {
            let portion = edge.weight();
            total_amount +=
                portion.amount * self.get_amount_(edge.target());
        }
        total_amount
    }

    fn get_price_(&self, index: NodeIndex) -> f64
    {
        let mut total_price = self.graph[index].borrow().price;
        for edge in self.graph.edges(index) {
            let portion = edge.weight();
            total_price +=
                portion.amount * self.get_unit_price(edge.target());
        }
        total_price
    }

    pub fn get_price(&self, id: Uuid) -> Result<f64>
    {
        let node = self.get_node(&id)?;
        let price = self.get_price_(*node);
        Ok(price)
    }

    pub fn get_amount(&self, id: Uuid) -> Result<f64>
    {
        let node = self.get_node(&id)?;
        let amount = self.get_amount_(*node);
        Ok(amount)
    }

    pub fn get_unit_price(&self, index: NodeIndex) -> f64
    {
        self.get_price_(index) / self.get_amount_(index)
    }
}

impl TryFrom<AppData> for Inventory
{
    type Error = anyhow::Error;

    fn try_from(data: AppData) -> Result<Self>
    {
        let mut inventory = Inventory::new();
        for item in data.items.into_iter() {
            inventory.add_item(item)?;
        }
        for portion in data.portions.into_iter() {
            inventory.add_portion(portion)?;
        }
        Ok(inventory)
    }
}
