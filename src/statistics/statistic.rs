use std::borrow::Borrow;

use chrono::prelude::*;
use js_sys::Array;
use serde_json::json;
use serde_wasm_bindgen::from_value;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlElement;
use web_sys::HtmlSelectElement;
use yew::prelude::*;
use yew_hooks::prelude::*;
use lazy_static::lazy_static;
use std::sync::Mutex;

#[path = "wpm.rs"]
mod wpm;
use wpm::calculate as calculate_wpm;

use crate::app::invoke;
use shared::FileWriteData;

#[derive(Properties, PartialEq)]
pub struct CharCountProps {
    pub closing_callback: Callback<MouseEvent>,
    pub pages_ref: NodeRef,
}


// #[derive(Properties, PartialEq)]
// pub struct StatisticProp {
//     pub char_count: usize,
//     pub pages_ref: NodeRef,
// }

lazy_static! {
    static ref START_TIME: Mutex<DateTime<Utc>> = Mutex::new(Utc::now());
}

// #[derive(Properties, PartialEq)]
// pub struct StatisticProp {
//     pub char_count: usize,
//     pub pages_ref: NodeRef,
// }

#[function_component]
pub fn Statistics(
    CharCountProps {
        closing_callback: on_close,
        pages_ref,
    }: &CharCountProps,
) -> Html {
    let char_count = use_state(|| 0);
    let char_count_no_spaces = use_state(|| 0);
    let word_count = use_state(|| 0);
    let session_time = use_state(|| String::from("00:00:00"));
    let start_time = use_state(Local::now);
    let calculated_wpm = calculate_wpm(*word_count, Some(*start_time));

    // Use an interval to update statistics every 1500 milliseconds
    {
        let char_count = char_count.clone();
        let char_count_no_spaces = char_count_no_spaces.clone();
        let word_count = word_count.clone();
        let session_time = session_time.clone();
        let pages_ref = pages_ref.clone();
        use_interval(
            {
                move || {
                    let char_count = char_count.clone();
                    let char_count_no_spaces = char_count_no_spaces.clone();
                    let word_count = word_count.clone();
                    let session_time = session_time.clone();
                    let start_time = start_time.clone();
                    let pages_ref = pages_ref.clone();
                    spawn_local(async move {
                        if let Some(pages_element) = pages_ref.cast::<HtmlElement>() {
                            // Locate the `notepad-textarea-edit` using query_selector
                            if let Ok(Some(notepad_element)) =
                                pages_element.query_selector("#notepad-textarea-edit")
                            {
                                let text = notepad_element.text_content().unwrap_or_default();

                                // Update character counts
                                let count = text.len();
                                let count_no_spaces =
                                    text.chars().filter(|c| !c.is_whitespace()).count();
                                char_count.set(count);
                                char_count_no_spaces.set(count_no_spaces);

                                // Update word count
                                let word_count_value = text.split_whitespace().count();
                                word_count.set(word_count_value);

                                // Update session time
                                let current_time = Local::now();
                                let session_duration = current_time - *start_time;
                                let total_seconds = session_duration.num_seconds();
                                let hours = total_seconds / 3600;
                                let minutes = (total_seconds % 3600) / 60;
                                let seconds = total_seconds % 60;
                                session_time.set(format!("{hours:02}:{minutes:02}:{seconds:02}"));

                                let json = json!({
                                    "session_time": (*session_time).clone(),
                                    "word_count": word_count_value,
                                    "char_count": count,
                                    "char_count_with_no_spaces": *char_count_no_spaces.clone(),
                                    "wpm": calculated_wpm
                                })
                                .to_string();

                                let start_time = *START_TIME.lock().unwrap();
                                let formatted_time = start_time.format("%Y-%m-%dT%H-%M-%S").to_string();

                                let path_jsvalue = invoke("get_data_dir", JsValue::null()).await;

                                let mut path_string =
                                    path_jsvalue.as_string().expect("Geming").clone();

                                path_string.push_str("/PaperSmith/");

                                let json_write = FileWriteData {
                                    path: path_string,
                                    name: formatted_time,
                                    content: json,
                                };

                                invoke(
                                    "write_to_json",
                                    serde_wasm_bindgen::to_value(&json_write).unwrap(),
                                )
                                .await;
                            }
                        }
                    });
                }
            },
            500,
        );
    }

    html! {
        <div>
            { format!("{}, {} Words; Characters: {}, {} without spaces, {:.2} wpm", *session_time, *word_count, *char_count,*char_count_no_spaces, calculated_wpm) }
        </div>
    }
}

#[function_component]
pub fn StatisticWindow(
    CharCountProps {
        closing_callback: on_close,
        pages_ref,
    }: &CharCountProps,
) -> Html {
    let files = use_state(|| vec![]);
    let selected_file = use_state(|| String::new());
    let formatted_file = use_state(|| String::new());

    {
        let files = files.clone();
        use_interval(move || {
            let files = files.clone();

            {
            spawn_local(async move {
            let file_list = invoke("list_statistic_files", JsValue::NULL).await;
            let file_list: Vec<String> = from_value(file_list).unwrap();
    
            files.set(file_list);
    
        })}}, 1000);
        }

    fn files_to_html(files: Vec<String>) -> Html {
        files
            .iter()
            .map(|file| {
                html! {
                    <option value={file.clone()}>
                        { file }
                    </option>
                }
            })
            .collect()
    }

    html! {
        <>
            <div class="absolute top-0 left-0 z-50 bg-black/60 h-full w-full flex items-center justify-center text-text">
                <div class="bg-base rounded-lg max-w-[50%] min-w-[50%] max-h-[70%] min-h-[70%] p-8 flex flex-col justify-between">
                    <div>
                        <label for="file-select" class="block text-white-700 font-medium mb-2">{"Select a file"}</label>
                        <select
                            id="file-select"
                            class="bg-base rounded-lg text-text focus:ring-secondary border-1 border-primary"
                            onchange={Callback::from(move |e: Event| {
                                let target = e.target_dyn_into::<web_sys::HtmlSelectElement>().unwrap();
                                selected_file.set(target.value());
                            })}
                        >
                            <option value="">{"-- Select --"}</option>
                            {
                                files_to_html((*files).clone())
                            }
                        </select>
                    </div>
    
                    <button
                        onclick={on_close}
                        class="rounded-lg text-lg px-2 py-1 bg-secondary text-crust hover:scale-105 border-0 transition-transform self-end"
                    >
                        { "Close" }
                    </button>
                </div>
            </div>
        </>
    }
}    