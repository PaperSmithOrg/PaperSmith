use chrono::prelude::*;
use gloo_timers::callback::Timeout;
use serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use serde_wasm_bindgen::from_value;
use wasm_bindgen::JsValue;
use web_sys::HtmlElement;
use web_sys::HtmlSelectElement;
use yew::platform::spawn_local;
use yew::prelude::*;
use yew_hooks::prelude::*;

#[path = "wpm.rs"]
mod wpm;
use wpm::calculate as calculate_wpm;

use crate::app::invoke;
use shared::FileWriteData;

#[derive(Properties, PartialEq)]
pub struct StatisticsProps {
    pub pages_ref: NodeRef,
}

#[derive(Properties, PartialEq)]
pub struct StatisticsWindowProps {
    pub closing_callback: Callback<MouseEvent>,
}

// #[derive(Properties, PartialEq)]
// pub struct StatisticProp {
//     pub char_count: usize,
//     pub pages_ref: NodeRef,
// }

#[derive(Serialize)]
pub struct PathArgs {
    pub name: String,
}
#[derive(Serialize)]
pub struct PathArgsJson {
    pub path: String,
}

// #[derive(Properties, PartialEq)]
// pub struct StatisticProp {
//     pub char_count: usize,
//     pub pages_ref: NodeRef,
// }

#[derive(Deserialize, Serialize, Debug)]
struct FileContent {
    char_count: u32,
    char_count_with_no_spaces: u32,
    session_time: String,
    word_count: u32,
    wpm: f32,
}

#[function_component]
pub fn Statistics(StatisticsProps { pages_ref }: &StatisticsProps) -> Html {
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

                                let formatted_time =
                                    start_time.format("%Y-%m-%dT%H-%M-%S").to_string();

                                let path_jsvalue = invoke("get_data_dir", JsValue::null()).await;

                                let mut path_string =
                                    path_jsvalue.as_string().expect("Geming").clone();

                                path_string.push_str("/PaperSmith/Statistics/");

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
    StatisticsWindowProps {
        closing_callback: on_close,
    }: &StatisticsWindowProps,
) -> Html {
    let files = use_state(Vec::new);
    let selected_file = use_state(String::new);
    let file_content = use_state(String::new);
    let select_ref = use_node_ref();

    let onchange = {
        let select_ref = select_ref.clone();
        let file_content = file_content.clone();

        Callback::from(move |_| {
            let select_ref = select_ref.clone();
            let selected_file = selected_file.clone();
            let file_content = file_content.clone();

            let _ = Timeout::new(10, {
                move || {
                    let select = select_ref.cast::<HtmlSelectElement>();

                    if let Some(select) = select {
                        let selected_file_value = select.value();

                        {
                            let file_content = file_content.clone();
                            spawn_local(async move {
                                let selected_file_value = selected_file_value.clone();
                                let file_content = file_content.clone();

                                let file_name = PathArgs {
                                    name: selected_file_value.clone(),
                                };

                                gloo_console::log!(format!("{}", selected_file_value));

                                let file_name_jsvalue = invoke(
                                    "unformat_file_name",
                                    serde_wasm_bindgen::to_value(&file_name).unwrap(),
                                )
                                .await;

                                let file_name = serde_wasm_bindgen::from_value::<Option<String>>(
                                    file_name_jsvalue,
                                )
                                .unwrap()
                                .unwrap();
                                gloo_console::log!(format!("{}", file_name));


                                let base_path_jsvalue = invoke("get_data_dir", JsValue::null()).await;

                                let mut base_path = serde_wasm_bindgen::from_value::<Option<String>>(
                                    base_path_jsvalue,
                                )
                                .unwrap()
                                .unwrap();

                                base_path.push_str("/PaperSmith/Statistics/");
                                base_path.push_str(&file_name);

                                //gloo_console::log!(format!("{}",path));

                                let path = PathArgsJson {
                                    path: base_path.clone(),
                                };

                                let file_content_jsvalue = invoke(
                                    "read_json_file",
                                    serde_wasm_bindgen::to_value(&path).unwrap(),
                                )
                                .await;

                                let temp_string: Option<String> = serde_wasm_bindgen::from_value(file_content_jsvalue).unwrap();
                                    if let Some(content) = temp_string {
                                        gloo_console::log!(format!("File Content: {}", content));

                                        let parsed_content: Result<FileContent, _> =
                                        serde_json::from_str(&content);

                                    if let Ok(parsed) = parsed_content {
                                        let display_content = format!(
                                            "Session Time: {}\nWords: {}\nCharacter Count: {}\nCharacters without spaces: {}\nWPM: {}",
                                            parsed.session_time,
                                            parsed.word_count,
                                            parsed.char_count,
                                            parsed.char_count_with_no_spaces,
                                            parsed.wpm
                                        );
                                        file_content.set(display_content);
                                    } else {
                                        gloo_console::log!(format!("Failed to parse JSON into FileContent struct."));
                                    }
                                } else {
                                    gloo_console::log!(format!("Failed to read file content."));
                                    file_content.set(String::from("Failed to load file content."));
                                }
                            });
                        }

                        selected_file.set(select.value());
                    }
                }
            })
            .forget();
        })
    };

    {
        let files = files.clone();
        use_interval(
            move || {
                let files = files.clone();

                {
                    spawn_local(async move {
                        let file_list = invoke("list_statistic_files", JsValue::NULL).await;
                        let file_list: Vec<String> = from_value(file_list).unwrap();

                        files.set(file_list);
                    });
                }
            },
            1000,
        );
    }

    html! {
        <>
            <div
                class="absolute top-0 left-0 z-50 bg-mantle/70 h-full w-full flex items-center justify-center text-text"
            >
                <div
                    class="bg-base rounded-lg max-w-[30%] min-w-[30%] max-h-[50%] min-h-[50%] p-8 flex flex-col justify-between"
                >
                    <div>
                        <label for="file-select" class="block text-text font-medium mb-2">
                            { "Select a file" }
                        </label>
                        <select
                            ref={select_ref}
                            class="bg-subtext rounded-lg text-mantle focus:ring-accent border border-primary p-2 w-full"
                            onchange={onchange}
                        >
                            { files_to_html(&files) }
                        </select>
                    </div>
                    <div class="mt-4">
                        <label class="block text-subtext font-medium mb-2">
                            { "File Content" }
                        </label>
                        <div class="bg-mantle p-4 rounded-lg">
                            { if (*file_content).is_empty() {
                               html! { <p class="text-subtext">{"No content to display."}</p> }
                            } else {
                               html! { <pre class="text-text whitespace-pre-wrap overflow-auto">{ &*file_content }</pre> }
                            } }
                        </div>
                    </div>
                    <button
                        onclick={on_close}
                        class="rounded-lg text-lg px-3 py-1 bg-secondary text-crust hover:bg-accent hover:scale-105 border-0 transition-transform self-end shadow-md"
                    >
                        { "Close" }
                    </button>
                </div>
            </div>
        </>
    }
}

fn files_to_html(files: &[String]) -> Html {
    files
        .iter()
        .map(|file| {
            html! { <option class="bg-mantle" value={file.clone()}>{ file }</option> }
        })
        .collect()
}
