use floem::{
    event::EventListener,
    peniko::Color,
    reactive::{
        create_effect, create_rw_signal, create_trigger, use_context,
        RwSignal,
    },
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
    si::{f64::Mass, mass::kilogram},
};
use uuid::Uuid;

use super::field_border_validation;
use crate::{
    clip_uuid, mass_format_logic1,
    model::item::Item,
    view::{
        text_to_value,
        validation::{
            amount_validation, name_validation, price_validation,
        },
    },
    AppMode, AppState as State,
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ViewItem
{
    pub id: Uuid,
    pub name: String,
    pub amount: String,
    pub price: String,
}

impl From<Item> for ViewItem
{
    fn from(item: Item) -> ViewItem
    {
        Self {
            id: item.id,
            name: item.name.to_string(),
            amount: mass_format_logic1(
                item.amount.value,
                DisplayStyle::Description,
            ),
            price: format!("R$ {}", item.price),
        }
    }
}

pub fn item_form() -> impl IntoView
{
    let state: RwSignal<State> = use_context().unwrap();
    //--- inner
    let name = create_rw_signal(None);
    let amount = create_rw_signal(None);
    let price = create_rw_signal(None);
    //--- outer
    let name_text = create_rw_signal(String::new());
    let amount_text = create_rw_signal(String::new());
    let price_text = create_rw_signal(String::new());
    //--- triggers
    // let commit = create_trigger();
    let clear = create_trigger();
    let delete = create_trigger();

    // create_effect(move |_| {
    //     commit.track();
    //     state.update(|state| {
    //         state.mode = AppMode::default();
    //     })
    // });
    create_effect(move |_| {
        clear.track();
        name_text.set("".into());
        amount_text.set("".into());
        price_text.set("".into());
        name.set(None);
        amount.set(None);
        price.set(None);
    });
    create_effect(move |_| {
        let state = state.get();
        let mode = state.mode;
        let inventory = state.model;
        if let AppMode::EditMode(id) = mode {
            if let Ok(item) = inventory.get_item(&id) {
                let item_name = item.borrow().name.to_string();
                let item_amount = item.borrow().amount;
                let item_price = item.borrow().price;

                name.set(Some(item_name.clone()));
                amount.set(Some(Mass::new::<kilogram>(item_amount.value)));
                price.set(Some(item_price));

                name_text.set(item_name);
                amount_text.set(mass_format_logic1(
                    item_amount.value,
                    DisplayStyle::Abbreviation,
                ));
                price_text.set(item_price.to_string());
            }
        }
    });
    create_effect(move |_| {
        delete.track();
        if let AppMode::EditMode(id) = state.get_untracked().mode {
            state.update(|state| {
                let inventory = &mut state.model;
                if let Err(e) = inventory.remove_component(&id) {
                    eprintln!("{:?}", e);
                }
                state.mode = AppMode::default();
            });
        }
    });

    //--- produce view
    v_stack((h_stack((
        v_stack((
            label(|| "nome"),
            text_input(name_text)
                .on_event_stop(
                    EventListener::FocusLost,
                    text_to_value(name_text, name_validation, name),
                )
                .style(field_border_validation(name)),
        )),
        v_stack((
            label(|| "qtd. "),
            text_input(amount_text)
                .on_event_stop(
                    EventListener::FocusLost,
                    text_to_value(amount_text, amount_validation, amount),
                )
                .style(field_border_validation(amount)),
        )),
        v_stack((
            label(|| "valor"),
            text_input(price_text)
                .on_event_stop(
                    EventListener::FocusLost,
                    text_to_value(price_text, price_validation, price),
                )
                .style(field_border_validation(price)),
        )),
        v_stack((
            dyn_container(
                move || state.get().mode,
                move |mode| match mode {
                    AppMode::EditMode(_) => button(|| "excluir")
                        .on_click_stop(move |_| delete.notify())
                        .into_any(),
                    _ => label(|| "").style(|s| s.height(25.0)).into_any(),
                },
            ),
            button(move || match state.get().mode {
                AppMode::InsertMode => "registrar",
                _ => "atualizar",
            })
            .on_click_stop(move |_| {
                use AppMode::*;
                let valid_item = || -> Option<Item> {
                    let mut item_ = &mut Item::builder();
                    let name = name.get()?;
                    let amount = amount.get()?;
                    let price = price.get()?;
                    #[allow(clippy::single_match)]
                    match state.get().mode {
                        EditMode(id) | PortionMode(id, _) => {
                            item_ = item_.with_id(id);
                            // commit.notify();
                        }
                        _ => {}
                    }
                    item_
                        .with_name(&name)
                        .with_amount(amount)
                        .with_price(price)
                        .build()
                        .ok()
                };
                if let Some(item) = valid_item() {
                    state.update(|state| {
                        // println!("~ {:?}", state.mode);
                        match state.mode {
                            InsertMode => {
                                state.model.add_item(item.clone())
                            }
                            EditMode(id) | PortionMode(id, _) => {
                                let update_result = state
                                    .model
                                    .update_item(id, item.clone());
                                state.mode = AppMode::default();
                                update_result
                            }
                        }
                        .unwrap();
                        // println!("{:?}", state.model.get_portions(item.id))
                    });
                } else {
                    eprintln!("failed to add item");
                }
                clear.notify();
            })
            .style(move |s| match state.get().mode {
                AppMode::InsertMode => {
                    s.border_color(Color::DARK_SLATE_GRAY)
                }
                _ => s.border_color(Color::DARK_GREEN),
            }),
        ))
        .style(|s| s.margin_left(10.0)),
    ))
    .style(move |s| s.flex_row().padding(5.0).margin(5.0)),))
}

pub fn item_list(selected: Option<Uuid>) -> impl IntoView
{
    let state: RwSignal<State> = use_context().unwrap();
    let list = create_rw_signal(im::Vector::<ViewItem>::new());

    create_effect(move |_| {
        let item_list: Vec<ViewItem> = state
            .get()
            .model
            .list_components()
            .into_iter()
            .filter_map(|item: Item| {
                if let Some(selected) = selected {
                    let circular = state
                        .get()
                        .model
                        .test_portion(item.id, selected, 1.0)
                        .unwrap_or(true);
                    // dbg!(circular, selected);
                    if !circular { Some(item.into()) } else { None }
                } else {
                    Some(item.into())
                }
            })
            .collect();

        list.set(item_list.into());
    });

    let virtual_list = virtual_list(
        VirtualDirection::Vertical,
        VirtualItemSize::Fixed(Box::new(|| 25.0)),
        move || list.get(),
        move |item: &ViewItem| item.clone(),
        move |item: ViewItem| {
            h_stack((
                label(move || -> String { clip_uuid(item.id, 5) })
                    .style(move |s| s.min_width(60.0)),
                label(move || item.name.clone())
                    .style(|s| s.min_width(90.0)),
                label(move || {
                    state
                        .get_untracked()
                        .model
                        .get_amount(item.id)
                        .unwrap_or(-1.0)
                })
                .style(|s| s.min_width(120.0)),
                label(move || {
                    state
                        .get_untracked()
                        .model
                        .get_price(item.id)
                        .unwrap_or(-1.0)
                })
                .style(|s| s.min_width(60.0)),
            ))
            .style(move |s| s.padding_top(5.0).padding_horiz(15.0))
        },
    );

    let selected = virtual_list.selection();
    let on_select = move |index: usize| {
        use crate::AppMode::*;
        // dbg!(index);
        if let Some(view_item) = list.get().get(index) {
            // dbg!(view_item);
            let id = view_item.id;
            if let Ok(component_ref) = state.get().model.get_item(&id) {
                let component = component_ref.borrow();
                match state.get_untracked().mode {
                    InsertMode => {
                        state.update(|state| {
                            state.mode = EditMode(component.id);
                        });
                    }
                    EditMode(src_id) => state.update(|state| {
                        state.mode = PortionMode(src_id, id);
                    }),
                    _ => {
                        unreachable!(
                            "item list should not be visible in \
                             `PortionMode`"
                        );
                    }
                }
            }
        }
    };
    container(
        scroll(
            virtual_list
                .style(move |s| s.flex_col().width_full().padding_top(5.0))
                .on_select(move |maybe_index| {
                    if let Some(index) = maybe_index {
                        selected.set(None);
                        on_select(index);
                        // println!("hm...");
                    }
                }),
        )
        .style(|s| s.width(100.pct()).height(100.pct())),
    )
    .style(|s| {
        s.height_full().padding_vert(15.0).flex_col().items_center()
    })
}
