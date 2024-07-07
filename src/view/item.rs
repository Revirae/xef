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
use uom::si::{f64::Mass, mass::kilogram};
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
            amount: mass_format_logic1(item.amount.value),
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
    let clear = create_trigger();
    let delete = create_trigger();

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
        let inventory = state.model.borrow();
        if let AppMode::EditMode(id) = mode {
            if let Ok(item) = inventory.get_item(&id) {
                let item_name = item.name.to_string();
                let item_amount = item.amount;
                let item_price = item.price;

                name.set(Some(item_name.clone()));
                amount.set(Some(Mass::new::<kilogram>(item_amount.value)));
                price.set(Some(item_price));

                name_text.set(item_name);
                amount_text.set(mass_format_logic1(item_amount.value));
                price_text.set(item_price.to_string());
            }
        }
    });
    create_effect(move |_| {
        delete.track();
        if let AppMode::EditMode(id) = state.get_untracked().mode {
            state.update(|state| {
                let mut model = state.model.borrow_mut();
                if let Err(e) = model.remove_item(&id) {
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
                        EditMode(src_id)
                        | PortionMode(src_id, _)
                        | EditPortionMode(src_id, _) => {
                            item_ = item_.with_id(src_id);
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
                    state.update(|state| match state.mode {
                        InsertMode => {
                            let mut model = state.model.borrow_mut();
                            model.add_item(item.clone()).unwrap();
                        }
                        EditMode(src_id)
                        | PortionMode(src_id, _)
                        | EditPortionMode(src_id, _) => {
                            let mut model = state.model.borrow_mut();
                            model
                                .update_item(src_id, |mut i| {
                                    *i = item.clone()
                                })
                                .unwrap();
                        }
                    });
                } else {
                    eprintln!("failed to add item");
                }
                state.update(|state| state.mode = AppMode::default());
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

pub fn item_list(maybe_id: Option<Uuid>) -> impl IntoView
{
    let state: RwSignal<State> = use_context().unwrap();
    let list = create_rw_signal(im::Vector::<ViewItem>::new());

    create_effect(move |_| {
        let s = state.get();
        let model = s.model.clone();
        let item_list: Vec<ViewItem> = model
            .borrow()
            .list_item()
            .into_iter()
            .filter_map(|item: Item| {
                if let Some(selected_id) = maybe_id {
                    let mut model = model.borrow().clone();
                    let circular = model
                        // .borrow_mut()
                        .test_portion(item.id, selected_id)
                        .unwrap_or(true);
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
                    let s = state.get_untracked();
                    let model = s.model.borrow();
                    let amount = model.get_amount(item.id).unwrap_or(-1.0);
                    mass_format_logic1(amount)
                })
                .style(|s| s.min_width(120.0)),
                label(move || {
                    let s = state.get_untracked();
                    let model = s.model.borrow();
                    let price = model.get_price(item.id).unwrap_or(-1.0);
                    format!("R$ {:.2}", price)
                })
                .style(|s| s.min_width(60.0)),
            ))
            .style(move |s| s.padding_top(5.0).padding_horiz(15.0))
        },
    );

    let selected = virtual_list.selection();
    let on_select = move |index: usize| {
        use crate::AppMode::*;
        if let Some(view_item) = list.get().get(index) {
            let id = view_item.id;
            let s = state.get();
            let model = s.model.borrow();
            if let Ok(item) = model.get_item(&id) {
                match state.get_untracked().mode {
                    InsertMode => {
                        state.update(|state| {
                            state.mode = EditMode(item.id);
                        });
                    }
                    EditMode(src_id) => state.update(|state| {
                        state.mode = PortionMode(src_id, id);
                    }),
                    PortionMode(_, _) | EditPortionMode(_, _) => {
                        unreachable!(
                            "`Item` list should not be visible in \
                             `Portion` modes"
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
                    }
                }),
        )
        .style(|s| s.width(100.pct()).height(100.pct())),
    )
    .style(|s| {
        s.height_full().padding_vert(15.0).flex_col().items_center()
    })
}
