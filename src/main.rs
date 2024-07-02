use anyhow::Result;
use floem::{
    event::EventListener,
    reactive::{
        create_rw_signal, provide_context, use_context,
        RwSignal,
    },
    views::Decorators,
};
use xef::{database::AppData, view::app_view, AppState};

static DBFN: &str = "xef.json";

fn persist() -> Result<()>
{
    let state: RwSignal<AppState> = use_context().unwrap();
    let data: AppData = state.get().model.into();
    data.save(DBFN)?;
    Ok(())
}

fn main() -> Result<()>
{
    let mut state = AppState::default();
    if let Ok(data) = AppData::load(DBFN) {
        state.set_data(data)?;
    }

    let state_handle = create_rw_signal(state);
    provide_context(state_handle);

    let main_view = || {
        app_view().on_event_stop(
            EventListener::WindowClosed,
            move |_| persist().unwrap(),
        )
    };

    floem::launch(main_view);
    Ok(())
}
