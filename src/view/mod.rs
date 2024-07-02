use anyhow::Result;
use floem::{
    event::Event,
    peniko::Color,
    reactive::{create_rw_signal, use_context, RwSignal},
    style::Style,
    views::{button, dyn_container, h_stack, label, v_stack, Decorators},
    IntoView,
};

pub mod item;
pub mod portion;
pub mod validation;
use self::item::{item_form, item_list};
use crate::{
    view::portion::{portion_form, portion_list},
    AppState as State,
};

fn text_to_value<T: 'static>(
    raw_text: RwSignal<String>,
    process: impl Fn(String) -> Result<T>,
    value: RwSignal<Option<T>>,
) -> impl Fn(&Event)
{
    move |_| {
        value.set(process(raw_text.get()).ok());
    }
}

//--- constants
const WRONG_COLOR: Color = Color::ORANGE_RED;
const RIGHT_COLOR: Color = Color::FOREST_GREEN;

//--- style methods
pub fn field_border_validation<T: Clone + 'static>(
    signal: RwSignal<Option<T>>,
) -> impl Fn(Style) -> Style + 'static
{
    move |s: Style| {
        let c = if signal.get().is_none() {
            WRONG_COLOR
        } else {
            RIGHT_COLOR
        };
        s.border_color(c).margin(5.0)
    }
}

#[derive(Clone, PartialEq)]
enum Page
{
    ItemPage,
}

fn main_page() -> impl IntoView
{
    use crate::AppMode::*;
    let state: RwSignal<State> = use_context().unwrap();
    v_stack((
        item_form(),
        dyn_container(
            move || state.get().mode,
            |mode| match mode {
                InsertMode => item_list(None).into_any(),
                EditMode(id) => item_list(Some(id)).into_any(),
                _ => label(|| "-").into_any(),
            },
        ),
        dyn_container(
            move || state.get().mode,
            |mode| match mode {
                EditMode(src_id) => portion_list(src_id).into_any(),
                PortionMode(src_id, id) => h_stack((
                    portion_form(src_id, id),
                    portion_list(src_id),
                ))
                .into_any(),
                _ => label(|| "-").into_any(),
            },
        ),
    ))
}

pub fn app_view() -> impl IntoView
{
    let page = create_rw_signal(Page::ItemPage);
    let state: RwSignal<State> = use_context().unwrap();

    v_stack((
        h_stack((button(move || format!("{:?}", state.get().mode))
            .on_click_stop(move |_| {
                page.set(Page::ItemPage);
            })
            .style(|s| s.margin_bottom(20)),)),
        dyn_container(
            move || page.get(),
            move |page| match page {
                Page::ItemPage => main_page().into_any(),
            },
        )
        .style(|s| s.padding(10).border(1).size_full()),
    ))
    .style(|s| {
        s.width_full()
            .height_full()
            .items_center()
            .justify_center()
            .row_gap(15)
    })
}
