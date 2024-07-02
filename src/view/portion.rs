// #![allow(unused)]
use anyhow::{anyhow, Result};
use floem::{
    event::EventListener,
    peniko::Color,
    reactive::{
        create_effect, create_rw_signal, create_signal, create_trigger,
        use_context, RwSignal,
    },
    style::Style,
    unit::UnitExt,
    views::{
        button, container, dyn_container, h_stack, label, scroll,
        text_input, v_stack, virtual_list, Decorators, VirtualDirection,
        VirtualItemSize,
    },
    IntoView,
};
use uom::{
    fmt::DisplayStyle,
    si::{
        f64::Mass,
        mass::{self, kilogram},
    },
};
use uuid::Uuid;

use super::{item::ViewItem, validation::amount_validation};
use crate::{
    clip_uuid, mass_format, mass_format_logic1,
    model::{
        inventory::{self, Inventory},
        item::Item,
        portion::Portion,
    },
    parse_mass_amount,
    view::text_to_value,
    AppMode, AppState as State,
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ViewPortion
{
    pub ingredient_name: String,
    pub amount: String,
    pub price: String,
}

pub fn portion_to_view(
    portion: Portion,
    inventory: &Inventory,
) -> Result<ViewPortion>
{
    let source = inventory.get_item(&portion.source_id)?;
    let component = inventory.get_item(&portion.component_id)?;

    let ingredient_name = component.borrow().name.to_string();
    let amount = portion.amount.to_string();
    let price = inventory.get_price(portion.source_id)? * portion.amount;
    let price = price.to_string();

    let view_portion = ViewPortion {
        ingredient_name,
        amount,
        price,
    };

    Ok(view_portion)
}

pub fn portion_form(src_id: Uuid, id: Uuid) -> impl IntoView
{
    let state: RwSignal<State> = use_context().unwrap();
    let (src_id, _) = create_signal(src_id);
    let (id, _) = create_signal(id);
    let item: RwSignal<Option<Item>> = create_rw_signal(None);
    let source: RwSignal<Option<Item>> = create_rw_signal(None);
    //--- inner
    let amount: RwSignal<Option<Mass>> = create_rw_signal(None);
    //--- outer
    let ing_name_text = create_rw_signal(String::new());
    let amount_text = create_rw_signal(String::new());
    let price_text = create_rw_signal(String::new());

    create_effect(move |_| {
        let inventory = &state.get_untracked().model;
        if let Some(amount) = amount.get() {
            if let Ok(node) = inventory.get_node(&src_id.get()) {
                let src_price = inventory.get_unit_price(*node);
                let price = amount.value * src_price;
                price_text.set(price.to_string());
            }
        }
    });

    create_effect(move |_| {
        let inventory = state.get_untracked().model;
        if let Ok(rc) = inventory.get_item(&src_id.get()) {
            let i = rc.borrow();
            source.set(Some(i.clone()));
        }
        if let Ok(rc) = inventory.get_item(&id.get()) {
            let i = rc.borrow();
            item.set(Some(i.clone()));
        }
    });
    create_effect(move |_| {
        if let Some(item) = item.get() {
            ing_name_text.set(item.name.to_string());
        }
    });

    v_stack((
        label(move || ing_name_text.get()),
        text_input(amount_text).on_event_stop(
            EventListener::FocusLost,
            text_to_value(amount_text, amount_validation, amount),
        ),
        label(move || price_text.get()),
        button(|| "Adicionar").on_click_stop(move |_| {
            let source_id = source.get().unwrap().id;
            let component_id = item.get().unwrap().id;
            if amount.get().is_none() {
                return;
            }
            let amount = amount.get().unwrap().value;

            state.update(|state| {
                let r = state.model.create_portion(
                    component_id,
                    source_id,
                    amount,
                );
            });
        }),
    ))
}

pub fn portion_list(id: Uuid) -> impl IntoView
{
    let state: RwSignal<State> = use_context().unwrap();
    let (id, _) = create_signal(id);
    let list = create_rw_signal(im::Vector::<ViewPortion>::new());

    create_effect(move |_| {
        // dbg!(id.get());
        let mut inventory = &mut state.get_untracked().model;
        if let Ok(portions) = inventory.get_portions(id.get()) {
            // dbg!(portions.clone());
            list.set(
                portions
                    .into_iter()
                    .filter_map(|portion: Portion| {
                        portion_to_view(portion, inventory).ok()
                    })
                    .collect(),
            );
        }
    });

    container(
        scroll(
            virtual_list(
                VirtualDirection::Vertical,
                VirtualItemSize::Fixed(Box::new(|| 40.0)),
                move || list.get(),
                move |portion: &ViewPortion| portion.clone(),
                move |portion| {
                    h_stack((
                        label(move || portion.ingredient_name.clone())
                            .style(|s| s.min_width(90.0)),
                        label(move || portion.amount.clone())
                            .style(|s| s.min_width(120.0)),
                        label(move || portion.price.clone())
                            .style(|s| s.min_width(60.0)),
                    ))
                    .style(move |s| s.padding_top(5.0).padding_horiz(15.0))
                },
            )
            .style(move |s| s.flex_col().width_full().padding_top(5.0)), // .on_select(move |i| {
                                                                         //     if let Some(index) = i {
                                                                         //         if let Some(view_item) =
                                                                         //             list.get_untracked().get(index)
                                                                         //         {
                                                                         //             // let id = view_item.id;
                                                                         //             if let Ok(component_ref) =
                                                                         //                 state
                                                                         //                     .get_untracked()
                                                                         //                     .model
                                                                         //                     .get_component(&id)
                                                                         //             {
                                                                         //                 let component =
                                                                         //                     component_ref.borrow();
                                                                         //                 // mode.set(
                                                                         //                 //     AppMode::EditMode(
                                                                         //                 //         component.id(),
                                                                         //                 //     ),
                                                                         //                 // );
                                                                         //                 // selected.notify();
                                                                         //             }
                                                                         //         }
                                                                         //     }
                                                                         // }),
        )
        .style(|s| s.width(100.pct()).height(100.pct())),
    )
    .style(|s| {
        // s.size(100.pct(), 100.pct())
        s.height_full().padding_vert(15.0).flex_col().items_center()
    })
}
