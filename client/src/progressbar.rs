use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ProgressProps {
    pub progress: u8,
    pub outer_class: Classes,
    pub inner_class: Classes,
}

#[rustfmt::skip::macros(html)]
#[function_component(Progressbar)]
pub fn progressbar(props: &ProgressProps) -> Html {
    let style = format!("width: {}%;", props.progress);
    html! {
        <div class={props.outer_class.clone()}>
            <div class={props.inner_class.clone()} style={style}></div>
        </div>
    }
}
