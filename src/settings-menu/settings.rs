use gloo::utils::document;
use serde_json::json;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::spawn_local;
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

    let on_confirm = {
        let on_close = on_close.clone();
        let theme = theme.clone();

        Callback::from(move |_| {
            let theme = theme.clone();

            spawn_local(async move {
                switch_theme(theme.clone());
                write_changes(theme).await;
            });

            on_close.emit(MouseEvent::new("Dummy").unwrap())
        })
    };

    let switch_ref = use_node_ref();

    let themes = [
        "Light".to_string(),
        "Light Dark".to_string(),
        "Medium".to_string(),
        "Dark".to_string(),
        "Very Dark".to_string(),
    ];

    let onchange = {
        let select_ref = switch_ref.clone();
        let theme = theme.clone();

        Callback::from(move |_| {
            let select = select_ref.cast::<HtmlSelectElement>();
            let theme = theme.clone();

            if let Some(select) = select {
                theme.set(select.value());
            }
        })
    };

    let themes_vec = themes_to_html(Vec::from(themes.clone()));

    html!(
        <>
            <div class="text-xl font-bold">{ "Settings" }</div>
            <br />
            <div id="theme_change" class="flex w-full pt-8 justify-between">
                <div class="font-bold self-center">{"Theme"}</div>
                // <ThemeSwitcher theme={theme}/>
                <div>
                <div>
                    <select ref={switch_ref}
                            onchange={onchange}
                            class="bg-base rounded-lg text-text focus:ring-secondary border-1 border-primary"
                    >
                        { themes_vec }
                    </select>
                </div>
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

fn themes_to_html(themes: Vec<String>) -> Html {
    themes
        .iter()
        .map(|theme| {
            html! {
                <option value={theme.clone()}
                >
                { theme }
                </option>
            }
        })
        .collect()
}

async fn write_changes(theme: UseStateHandle<String>) {
    let content = json!({
        "theme": *theme,
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
}

fn switch_theme(theme: UseStateHandle<String>) {
    let html_doc: HtmlDocument = document().dyn_into().unwrap();
    let body = html_doc.body().unwrap();
    let theme = theme.as_str();
    let theme2 = theme.to_lowercase().replace(' ', "");
    body.set_class_name(format!("{theme2} bg-crust text-text").as_str());
}
