// #![allow(unused)]
use anyhow::Result;
use floem::{
    event::EventListener,
    reactive::{
        create_effect, create_memo, create_rw_signal, create_signal,
        create_trigger, use_context, RwSignal,
    },
    unit::UnitExt,
    views::{
        button, container, dyn_container, h_stack, label, scroll,
        text_input, v_stack, virtual_list, Decorators, VirtualDirection,
        VirtualItemSize,
    },
    IntoView,
};
use uom::si::f64::Mass;
use uuid::Uuid;

use super::validation::amount_validation;
use crate::{
    mass_format_logic1,
    model::{inventory::Inventory, item::Item, portion::Portion},
    view::text_to_value,
    AppMode, AppState as State,
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ViewPortion
{
    pub component_id: Uuid,
    pub ingredient_name: String,
    pub amount: String,
    pub price: String,
}

pub fn portion_to_view(
    portion: Portion,
    inventory: &Inventory,
) -> Result<ViewPortion>
{
    let component_id = portion.component_id;
    let item = inventory.get_item(&component_id)?;
    let ingredient_name = item.name.to_string();
    let amount = mass_format_logic1(portion.amount);
    let price = inventory.get_price(portion.source_id)? * portion.amount;
    let price = format!("R$ {:.2}", price);

    let view_portion = ViewPortion {
        component_id,
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
    //--- triggers
    let delete = create_trigger();

    create_effect(move |_| {
        let inventory = &state.get_untracked().model;
        if let Some(amount) = amount.get() {
            if let Ok(item_index) = inventory.get_node(&id.get()) {
                let src_price = inventory.get_unit_price(*item_index);
                let price = amount.value * src_price;
                price_text.set(price.to_string());
            }
        }
    });

    create_effect(move |_| {
        let inventory = state.get_untracked().model;
        if let Ok(i) = inventory.get_item(&src_id.get()) {
            source.set(Some(i));
        }
        if let Ok(i) = inventory.get_item(&id.get()) {
            item.set(Some(i));
        }
    });
    create_effect(move |_| {
        if let Some(item) = item.get() {
            ing_name_text.set(item.name.to_string());
        }
    });

    create_effect(move |_| {
        // delete.track();
        dbg!("###");
        let inventory = &mut state.get_untracked().model;
        if let AppMode::EditPortionMode(src_id, id) =
            state.get_untracked().mode
        {
            dbg!("!!!");
            state.update(|state| {
                if let Err(e) = inventory.remove_portion(id, src_id) {
                    eprintln!("{:?}", e);
                }
                state.mode = AppMode::EditMode(src_id);
            });
        }
    });

    v_stack((
        label(move || ing_name_text.get()),
        text_input(amount_text).on_event_stop(
            EventListener::FocusLost,
            text_to_value(amount_text, amount_validation, amount),
        ),
        label(move || price_text.get()),
        dyn_container(
            move || state.get().mode,
            move |mode| match mode {
                AppMode::EditPortionMode(_, _) => button(|| "excluir")
                    .on_click_stop(move |_| {
                        dbg!("clicked");
                        delete.notify()
                    })
                    .into_any(),
                _ => label(|| "").style(|s| s.height(25.0)).into_any(),
            },
        ),
        button(|| "Adicionar").on_click_stop(move |_| {
            let source_id = source.get().unwrap().id;
            let item_id = item.get().unwrap().id;
            if amount.get().is_none() {
                return;
            }
            let amount = amount.get().unwrap().value;

            state.update(|state| {
                state
                    .model
                    .create_portion(item_id, source_id, amount)
                    .unwrap();
            });
        }),
    ))
}

pub fn portion_list(src_id: Uuid) -> impl IntoView
{
    let state: RwSignal<State> = use_context().unwrap();
    let (src_id, _) = create_signal(src_id);
    // let (id, _) = create_signal(id);
    let list = create_rw_signal(im::Vector::<ViewPortion>::new());

    create_effect(move |_| {
        let inventory = &mut state.get_untracked().model;
        if let Ok(portions) = inventory.get_portions(src_id.get()) {
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
            .style(move |s| s.flex_col().width_full().padding_top(5.0))
            .on_select(move |i| {
                if let Some(index) = i {
                    if let Some(view_portion) =
                        list.get_untracked().get(index)
                    {
                        let id = view_portion.component_id;
                        if let Ok(_portion) = state
                            .get_untracked()
                            .model
                            .get_portion(id, src_id.get())
                        {
                            dbg!(_portion);
                            state.update(|state| {
                                state.mode = AppMode::EditPortionMode(
                                    src_id.get(),
                                    id,
                                )
                            });
                            // selected.notify();
                        }
                    }
                }
            }),
        )
        .style(|s| s.width(100.pct()).height(100.pct())),
    )
    .style(|s| {
        // s.size(100.pct(), 100.pct())
        s.height_full().padding_vert(15.0).flex_col().items_center()
    })
}
