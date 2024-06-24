use protocol::util::Percent;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ProgressProps {
    pub progress: Percent,
    #[prop_or_default]
    pub text: Option<String>,
    #[prop_or_default]
    pub text_class: Option<Classes>,
    pub outer_class: Classes,
    pub inner_class: Classes,
}

#[rustfmt::skip::macros(html)]
#[function_component(Progressbar)]
pub fn progressbar(props: &ProgressProps) -> Html {
    let style = format!("width: {}%;", props.progress.as_f64());
    html! {
        <div class={props.outer_class.clone()}>
            <div class={props.inner_class.clone()} style={style}></div>
            if props.text.is_some() {
                <div class={props.text_class.clone()}>{props.text.as_ref().unwrap()}</div>
            }
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct ProgressInteractiveProps {
    pub progress: Percent,
    pub outer_class: Classes,
    pub inner_class: Classes,
    pub on_slide: Callback<Percent>,
    pub disabled: bool,
}

#[rustfmt::skip::macros(html)]
#[function_component(ProgressbarInteractive)]
pub fn progressbar_interactive(props: &ProgressInteractiveProps) -> Html {
    let down = use_state_eq(|| false);
    let seek = use_state_eq(|| Percent::ZERO);

    {
        let on_slide = props.on_slide.clone();
        use_effect_with((down.clone(), seek.clone()), move |(down, seek)| {
            if **down {
                on_slide.emit(**seek);
            }
        });
    }

    let oninput = {
        let seek = seek.clone();
        let disabled = props.disabled;
        Callback::from(move |ie: InputEvent| {
            if disabled {
                return;
            }

            let input = ie
                .target()
                .and_then(|target| target.dyn_into().ok())
                .map(|ele: HtmlInputElement| ele.value());

            let Some(input) = input else {
                log::error!("Could not get value from range input");
                return;
            };

            log::trace!("Sliding {input:?}");

            let input: f64 = match input.parse() {
                Ok(f) => f,
                Err(e) => {
                    log::error!("Failed to parse input '{input}' as f64: {e}");
                    return;
                }
            };

            if let Some(p) = Percent::try_new(input) {
                seek.set(p);
            }
        })
    };

    let ontouchstart = {
        let down = down.setter();
        let seek = seek.setter();
        let disabled = props.disabled;
        let propsprog = props.progress;
        Callback::from(move |_| {
            if disabled {
                return;
            }
            log::trace!("Touch start");
            down.set(true);
            seek.set(propsprog);
        })
    };

    let ontouchend = {
        let down = down.setter();
        let disabled = props.disabled;
        Callback::from(move |_| {
            if disabled {
                return;
            }
            log::trace!("Touch end");
            down.set(false);
        })
    };

    let ontouchcancel = {
        let down = down.setter();
        let disabled = props.disabled;
        Callback::from(move |_| {
            if disabled {
                return;
            }
            log::trace!("Touch cancel");
            down.set(false);
        })
    };

    let onmousedown = {
        let down = down.setter();
        let seek = seek.setter();
        let disabled = props.disabled;
        let propsprog = props.progress;
        Callback::from(move |_| {
            if disabled {
                return;
            }
            log::trace!("Mouse down");
            down.set(true);
            seek.set(propsprog);
        })
    };

    let onmouseup = {
        let down = down.setter();
        let disabled = props.disabled;
        Callback::from(move |_| {
            if disabled {
                return;
            }
            log::trace!("Mouse up");
            down.set(false);
        })
    };

    let progress = if *down { *seek } else { props.progress };

    html! {
        <div class={props.outer_class.clone()}>
            <input type="range"
                   disabled={props.disabled}
                   min="0.0"
                   max="100.0"
                   step="any"
                   value={progress.as_f64().to_string()}
                   class={props.inner_class.clone()}
                   oninput={oninput}
                   // NOTE: the pointer events, which are an abstraction over input
                   // devices, didn't work well on my phone
                   ontouchstart={ontouchstart}
                   ontouchend={ontouchend}
                   ontouchcancel={ontouchcancel}
                   onmousedown={onmousedown}
                   onmouseup={onmouseup}
            />
        </div>
    }
}
