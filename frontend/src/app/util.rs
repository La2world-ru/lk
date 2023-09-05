use web_sys::{Event, HtmlInputElement, HtmlSelectElement, InputEvent};
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub value: String,
    pub on_change: Callback<String>,
}

pub fn get_value_from_input_event(e: InputEvent) -> String {
    let event: Event = e.dyn_into().unwrap_throw();
    let event_target = event.target().unwrap_throw();
    let target: HtmlInputElement = event_target.dyn_into().unwrap_throw();

    target.value()
}

pub fn get_value_from_event(event: Event) -> String {
    let event_target = event.target().unwrap_throw();
    let target: HtmlSelectElement = event_target.dyn_into().unwrap_throw();

    target.value()
}