use std::{future::Future, process::Output};

use gloo::utils::document;
use serde::{Deserialize, Serialize};
use serde_json::json;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlDocument;
use yew::prelude::*;

use shared::Settings;

use crate::app::invoke;

#[path = "switcher.rs"]
mod switcher;
use switcher::ThemeSwitcher;

#[derive(Properties, PartialEq)]
pub struct SettingsProps {
    pub closing_callback: Callback<MouseEvent>,
}

#[derive(Serialize, Deserialize)]
pub struct FileWriteData {
    pub path: String,
    pub name: String,
    pub content: String,
}

#[function_component(SettingsMenu)]
pub fn settings_menu(
    SettingsProps {
        closing_callback: on_close,
    }: &SettingsProps,
) -> Html {
    let confirm_button_ref = use_node_ref();

    let theme = use_state_eq(|| String::from("Light"));

    let on_confirm = {
        let on_close = on_close.clone();
        let theme = theme.clone();

        Callback::from(move |_| {
            let theme = theme.clone();

            switch_theme(theme.clone());

            spawn_local(async move {
                write_changes(theme).await;
            });

            on_close.emit(MouseEvent::new("Dummy").unwrap())
        })
    };

    html!(
        <>
            <div class="text-xl font-bold">{ "Settings" }</div>
            <br />
            // <div id="font_size_change" class="flex w-full pt-8 justify-between">
            //     <div class="font-semibold self-center">{ "Font Size" }</div>
            //     <div class="rounded-lg border-transparent hover:border-mauve">
            //         <input
            //             oninput={on_font_size_input}
            //             class="outline-none bg-crust p-2 rounded-lg border-2 border-transparent"
            //             ref={input_font_size_ref}
            //         />
            //     </div>
            // </div>
            <div id="theme_change" class="flex w-full pt-8 justify-between">
                <div class="font-bold self-center">{"Theme"}</div>
                <ThemeSwitcher theme={theme}/>
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

// fn field_input_handler(value: UseStateHandle<String>) -> Callback<InputEvent> {
//     Callback::from(move |ev: InputEvent| {
//         if let Some(input) = ev.target_dyn_into::<HtmlInputElement>() {
//             let text = input.value();

//             match text.clone().parse::<u32>() {
//                 Ok(n) => {
//                     gloo_console::log!(format!("text is {:?}", n));
//                     let _ = input.style().set_property("border-color", "transparent");
//                     value.set(text)
//                 }
//                 Err(err) => {
//                     gloo_console::log!(format!(
//                         "could not convert '{}' to number: {:?}",
//                         text, err
//                     ));

//                     let _ = input.style().set_property("border-color", "red");
//                 }
//             }
//         }
//     })
// }

async fn write_changes(theme: UseStateHandle<String>) {
    let content = json!({
        "theme": *theme,
    })
    .to_string();

    let name = String::from("settings");

    let path_jsvalue = invoke("get_data_dir", JsValue::null()).await;

    let mut path = path_jsvalue.as_string().expect("Cast failed").clone();

    path.push_str("/PaperSmith/");

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
    let theme = String::from(theme.clone().as_str());
    gloo_console::log!(theme.clone());
    let theme2 = theme.to_string().to_lowercase().replace(' ', "");
    body.set_class_name(format!("{theme2} bg-crust text-text").as_str());
}
