use std::ops::Deref;

use anyhow::Result;
use floem::{
    event::{Event, EventListener},
    reactive::{create_rw_signal, provide_context, use_context, RwSignal},
    views::Decorators,
};
use xef::{database::AppData, view::app_view, AppState};

static DBFN: &str = "xef.json";

fn try_persist() -> Result<()>
{
    let state: RwSignal<AppState> = use_context().unwrap();
    let model = state.get().model.borrow().deref().clone();
    let data: AppData = model.into();
    data.save(DBFN)
}

fn shutdown(_event: &Event) -> ()
{
    try_persist().expect("failed to persist");
}

fn try_init() -> Result<()>
{
    let mut state = AppState::default();
    if let Ok(data) = AppData::load(DBFN) {
        state.set_data(data)?;
    }

    let state_handle = create_rw_signal(state);
    provide_context(state_handle);
    Ok(())
}

fn main() -> Result<()>
{
    try_init()?;

    floem::launch(|| {
        app_view().on_event_stop(EventListener::WindowClosed, shutdown)
    });

    Ok(())
}
