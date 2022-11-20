use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ProgressProps {
    pub progress: f64,
    pub text: Option<String>,
    pub text_class: Option<Classes>,
    pub outer_class: Classes,
    pub inner_class: Classes,
}

#[rustfmt::skip::macros(html)]
#[function_component(Progressbar)]
pub fn progressbar(props: &ProgressProps) -> Html {
    assert!(props.progress >= 0.0 && props.progress <= 100.0);
    let style = format!("width: {}%;", props.progress);
    html! {
        <div class={props.outer_class.clone()}>
            <div class={props.inner_class.clone()} style={style}></div>
            if props.text.is_some() {
                <div class={props.text_class.clone()}>{props.text.as_ref().unwrap()}</div>
            }
        </div>
    }
}
