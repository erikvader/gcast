use gloo_events::EventListener;
use yew::hook;
use yew::use_effect_with;
use yew::use_state_eq;

#[hook]
pub fn use_page_visibility() -> bool {
    let visible = use_state_eq(|| true);

    {
        let visible = visible.clone();
        use_effect_with((), move |_| {
            let window = web_sys::window().expect("could not access window");
            let document = window.document().expect("could not access document");

            let cookie = EventListener::new(&window, "visibilitychange", move |_| {
                let visibility = document.visibility_state();
                log::info!("Page visibility change to: {:?}", visibility);
                visible.set(visibility == web_sys::VisibilityState::Visible);
            });

            move || drop(cookie)
        });
    }

    return *visible;
}
