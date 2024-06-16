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
    let seek = use_state_eq(|| props.progress);

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

    let onmousedown = {
        let down = down.setter();
        let disabled = props.disabled;
        Callback::from(move |_| {
            if disabled {
                return;
            }
            log::trace!("Down");
            down.set(true);
        })
    };

    let onmouseup = {
        let down = down.setter();
        let disabled = props.disabled;
        Callback::from(move |_| {
            if disabled {
                return;
            }
            log::trace!("Up");
            down.set(false);
        })
    };

    let progress = if *down { *seek } else { props.progress };

    html! {
        <div class={props.outer_class.clone()}>
            <input type="range"
                   min="0.0"
                   max="100.0"
                   step="any"
                   value={progress.as_f64().to_string()}
                   class={props.inner_class.clone()}
                   oninput={oninput}
                   onpointerdown={onmousedown}
                   onpointerup={onmouseup.clone()}
                   onpointercancel={onmouseup}
                   disabled={props.disabled}
            />
        </div>
    }
}
