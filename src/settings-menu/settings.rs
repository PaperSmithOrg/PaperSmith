use gloo::utils::document;
use serde::Serialize;
use serde_json::json;
use shared::Settings;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::HtmlDocument;
use web_sys::HtmlSelectElement;
use yew::platform::spawn_local;
use yew::prelude::*;
use yewdux::prelude::*;

use crate::app::invoke;
use crate::app::State;

use shared::FileWriteData;

#[derive(Properties, PartialEq)]
pub struct SettingsProps {
    pub closing_callback: Callback<MouseEvent>,
}

#[derive(Serialize)]
pub struct LogArgs {
    pub msg: String,
}

#[function_component(SettingsMenu)]
pub fn settings_menu(
    SettingsProps {
        closing_callback: on_close,
    }: &SettingsProps,
) -> Html {
    let (state, dispatch) = use_store::<State>();

    let confirm_button_ref = use_node_ref();

    let on_confirm = {
        let on_close = on_close.clone();
        let state = state.clone();

        Callback::from(move |_| {
            let state = state.clone();

            spawn_local(async move {
                let settings = state
                    .settings
                    .clone()
                    .unwrap_or_else(|| Settings::default());

                switch_theme(settings.theme.clone());

                let content = json!({
                    "theme": settings.theme,
                    "interval": settings.interval,
                })
                .to_string();

                let name = String::from("settings");

                let path_jsvalue = invoke("get_data_dir", JsValue::null()).await;

                let mut path = path_jsvalue.as_string().expect("Cast failed").clone();

                path.push_str("/PaperSmith");

                gloo_console::log!(path.clone());

                let settings = FileWriteData {
                    path,
                    name,
                    content,
                };

                invoke(
                    "write_to_json",
                    serde_wasm_bindgen::to_value(&settings).unwrap(),
                )
                .await;
            });

            on_close.emit(MouseEvent::new("Dummy").unwrap());
        })
    };

    let switch_ref = use_node_ref();

    let interval_ref = use_node_ref();

    let themes = [
        "Light".to_string(),
        "Light Dark".to_string(),
        "Medium".to_string(),
        "Dark".to_string(),
        "Very Dark".to_string(),
    ];

    let intervals = [0, 1, 3, 5, 10, 15, 30];

    let onchange = {
        let state = state.clone();
        let dispatch = dispatch.clone();
        let select_ref = switch_ref.clone();

        Callback::from(move |_| {
            let state = state.clone();
            let dispatch = dispatch.clone();
            let select = select_ref.cast::<HtmlSelectElement>();
            let settings = state
                .settings
                .clone()
                .unwrap_or_else(|| Settings::default());

            if let Some(select) = select {
                let value = select.value();

                let prev = settings.theme;

                let mut temp_settings = state.settings.as_ref().unwrap().clone();

                temp_settings.theme = value.clone();

                dispatch.reduce_mut(|state| state.settings = Some(temp_settings));

                spawn_local(async move {
                    let msg = LogArgs {
                        msg: format!("{prev:?} -> {value:?}"),
                    };
                    gloo_console::log!(format!("{value:?}"));
                    invoke("log", serde_wasm_bindgen::to_value(&msg).unwrap()).await;
                });
            }
        })
    };

    let on_interval_change = {
        let state = state.clone();
        let dispatch = dispatch.clone();
        let select_ref = interval_ref.clone();

        Callback::from(move |_| {
            let state = state.clone();
            let dispatch = dispatch.clone();
            let select = select_ref.cast::<HtmlSelectElement>();
            let settings = state
                .settings
                .clone()
                .unwrap_or_else(|| Settings::default());

            if let Some(select) = select {
                let value = select.value();

                let prev = settings.interval;

                let mut temp_settings = state.settings.as_ref().unwrap().clone();

                temp_settings.interval = select.value().parse::<u32>().unwrap() * 60 * 1000;

                dispatch.reduce_mut(|state| state.settings = Some(temp_settings));

                spawn_local(async move {
                    let msg = LogArgs {
                        msg: format!("{prev:?} -> {value:?}"),
                    };
                    gloo_console::log!(format!("{value:?}"));
                    invoke("log", serde_wasm_bindgen::to_value(&msg).unwrap()).await;
                });
            }
        })
    };

    let settings = state
        .settings
        .clone()
        .unwrap_or_else(|| Settings::default());

    let themes_vec = themes_to_html(&themes, settings.theme.clone());

    let interval_vec = get_intervals(&intervals, settings.interval.clone());

    html!(
        <>
            <div class="text-xl font-bold">{ "Settings" }</div>
            <br />
            <div id="theme_change" class="flex w-full pt-8 justify-between">
                <div class="font-bold self-center">{ "Theme" }</div>
                // <ThemeSwitcher theme={theme}/>
                <div>
                        <select
                            ref={switch_ref}
                            onchange={onchange}
                            class="bg-base rounded-lg text-text focus:ring-secondary border-1 border-primary"
                        >
                            { themes_vec }
                        </select>
                </div>
            </div>
            <br />
            <div id="interval_change" class="flex w-full pt-8 justify-between">
                <div class="font-bold self-center">{ "Auto-Save Interval" }</div>
                // <ThemeSwitcher theme={theme}/>
                    <div>
                        <select
                            ref={interval_ref}
                            onchange={on_interval_change}
                            class="bg-base rounded-lg text-text focus:ring-secondary border-1 border-primary"
                        >
                            { interval_vec }
            </select>
                </div>
            </div>
            <div class="flex justify-end w-full pt-8">
                <button
                    ref={confirm_button_ref}
                    onclick={on_confirm}
                    class="rounded-lg text-lg px-2 py-1 ml-4 bg-primary text-crust hover:scale-105 border-0"
                >
                    { "Confirm" }
                </button>
                <button
                    onclick={on_close}
                    class="rounded-lg text-lg px-2 py-1 ml-4 bg-secondary text-crust hover:scale-105 border-0"
                >
                    { "Close" }
                </button>
            </div>
        </>
    )
}

fn themes_to_html(themes: &[String], current: String) -> Html {
    themes
        .iter()
        .map(|theme| {
            let selected = current == *theme;
            gloo_console::log!(format!("{theme:?} is {selected:?}"));
            html! { <option value={theme.clone()} selected={selected}>{ theme }</option> }
        })
        .collect()
}

fn get_intervals(intervals: &[i32], current: u32) -> Html {
    intervals
        .iter()
        .map(|interval| {
            let time = *interval as u32;
            let selected = current == time * 60 * 1_000;
            html! { <option value={ interval.to_string()} selected={selected}>{ interval.to_string() + "min" }</option> }
        })
        .collect()
}

fn switch_theme(theme: String) {
    let html_doc: HtmlDocument = document().dyn_into().unwrap();
    let body = html_doc.body().unwrap();
    let theme = theme.as_str();
    let theme2 = theme.to_lowercase().replace(' ', "");
    body.set_class_name(format!("{theme2} bg-crust text-text").as_str());
}
