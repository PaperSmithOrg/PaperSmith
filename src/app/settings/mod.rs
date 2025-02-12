use gloo::utils::document;
use serde_json::json;
use wasm_bindgen::{JsCast, JsValue};
use yew::platform::spawn_local;
use web_sys::HtmlDocument;
use web_sys::HtmlSelectElement;
use yew::prelude::*;

use crate::app::invoke;

use shared::FileWriteData;

#[derive(Properties, PartialEq)]
pub struct SettingsProps {
    pub closing_callback: Callback<MouseEvent>,
}

#[function_component(SettingsMenu)]
pub fn settings_menu(
    SettingsProps {
        closing_callback: on_close,
    }: &SettingsProps,
) -> Html {
    let confirm_button_ref = use_node_ref();

    let theme = use_state(|| String::from("Light"));

    let interval = use_state(|| 300_000);

    let on_confirm = {
        let on_close = on_close.clone();
        let theme = theme.clone();
        let interval = interval.clone();

        Callback::from(move |_| {
            let theme = theme.clone();
            let interval = interval.clone();

            spawn_local(async move {
                switch_theme(&theme);

                let content = json!({
                    "theme": *theme,
                    "interval": *interval,
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

    let onchange = {
        let select_ref = switch_ref.clone();

        Callback::from(move |_| {
            let select = select_ref.cast::<HtmlSelectElement>();
            let theme = theme.clone();

            if let Some(select) = select {
                theme.set(select.value());
            }
        })
    };

    let on_interval_change = {
        let select_ref = interval_ref.clone();

        Callback::from(move |_| {
            let select = select_ref.cast::<HtmlSelectElement>();
            let interval = interval.clone();

            if let Some(select) = select {
                interval.set(select.value().parse::<i32>().unwrap() * 60 * 1_000);
            }
        })
    };

    let themes_vec = themes_to_html(&themes);

    let interval_vec = get_intervals();

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

fn themes_to_html(themes: &[String]) -> Html {
    themes
        .iter()
        .map(|theme| {
            html! { <option value={theme.clone()}>{ theme }</option> }
        })
        .collect()
}

fn get_intervals() -> Html {
    let intervals = [0, 1, 3, 5, 10, 15, 30];

    intervals
        .iter()
        .map(|interval| {
            html! { <option value={ interval.to_string() }>{ interval.to_string() + "min" }</option> }
        })
        .collect()
}

fn switch_theme(theme: &UseStateHandle<String>) {
    let html_doc: HtmlDocument = document().dyn_into().unwrap();
    let body = html_doc.body().unwrap();
    let theme = theme.as_str();
    let theme2 = theme.to_lowercase().replace(' ', "");
    body.set_class_name(format!("{theme2} bg-crust text-text").as_str());
}
