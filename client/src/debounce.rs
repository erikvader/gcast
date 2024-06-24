use gloo_timers::callback::Timeout;
use std::time::Duration;
use yew::prelude::*;

#[hook]
pub fn use_debounce<IN, F>(timeout: Duration, f: F) -> Callback<IN>
where
    F: Fn(IN) + 'static,
    IN: 'static,
{
    let timer = use_mut_ref(|| None::<Timeout>);
    let f = use_callback((), move |args, ()| f(args));

    let callback = {
        // NOTE: not sure if these many copies are necessary
        let timer = timer.clone();
        let f = f.clone();
        use_callback((), move |args, ()| {
            let old = timer.replace(None);
            if let Some(old) = old {
                old.cancel();
            }
            let f = f.clone();
            *timer.borrow_mut() =
                Some(Timeout::new(timeout.as_millis() as u32, move || {
                    f.emit(args)
                }));
        })
    };

    {
        let timer = timer.clone();
        use_effect_with((), move |()| {
            move || {
                if let Some(old) = timer.replace(None) {
                    old.cancel();
                }
            }
        });
    }

    callback
}
