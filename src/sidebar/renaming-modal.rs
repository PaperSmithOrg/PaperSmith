use std::path::PathBuf;

use serde::Serialize;
use serde_wasm_bindgen::to_value;
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlButtonElement, HtmlInputElement};
use yew::prelude::*;
use yewdux::prelude::*;

use crate::app::{invoke, modal::Modal, PathArgs, State};

#[derive(Serialize)]
struct RenameArgs {
    path: PathBuf,
    old: String,
    new: String,
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub old_name: String,
    pub closing_callback: Callback<MouseEvent>,
    pub is_project: bool,
}

#[function_component(RenamingModal)]
pub fn renamingmodal(
    Props {
        old_name,
        closing_callback: on_close,
        is_project,
    }: &Props,
) -> Html {
    let (state, dispatch) = use_store::<State>();
    let new_name_ref = use_node_ref();
    let confirm_ref = use_node_ref();
    let new_name = use_state(String::new);
    let is_data_valid = use_state(|| true);
    let error_message = use_state(String::new);

    let new_name_input = text_input_handler(new_name.clone());

    {
        let old_name = old_name.clone();
        let new_name = new_name.clone();
        let new_name_ref = new_name_ref.clone();
        use_effect_with((), move |()| {
            if let Some(input) = new_name_ref.cast::<HtmlInputElement>() {
                new_name.set(old_name.clone());
                input.set_value(&old_name);
            }
        });
    }

    {
        let is_data_valid = is_data_valid.clone();
        let new_name = new_name.clone();
        let state = state.clone();
        let error_message = error_message.clone();
        let is_project = *is_project;
        use_effect_with(new_name.clone(), move |_| {
            let is_data_valid = is_data_valid.clone();
            let new_name = new_name.clone();
            let error_message = error_message.clone();
            if new_name.is_empty() {
                is_data_valid.set(false);
                error_message.set("Please enter a new name.".to_string());
                return;
            }
            spawn_local(async move {
                let is_data_valid = is_data_valid.clone();
                let error_message = error_message.clone();

                let mut complete_path = PathBuf::from(&state.project.as_ref().unwrap().path);
                if is_project {
                    complete_path.pop();
                } else {
                    complete_path.push("Chapters");
                }
                complete_path.push(&*new_name);

                let result = invoke(
                    "can_create_path",
                    serde_wasm_bindgen::to_value(&PathArgs {
                        path: complete_path.into_os_string().into_string().unwrap(),
                    })
                    .unwrap(),
                )
                .await
                .as_string()
                .expect("Something went horribly wrong in validation");
                let (is_valid, message) = match result.as_str() {
                    "" => (true, ""),
                    e => (false, e),
                };

                is_data_valid.set(is_valid);
                error_message.set(message.to_string());
            });
        });
    }

    {
        let confirm_ref = confirm_ref.clone();
        let is_data_valid = is_data_valid.clone();
        use_effect_with(is_data_valid.clone(), move |_| {
            if let Some(button) = confirm_ref.cast::<HtmlButtonElement>() {
                if *is_data_valid {
                    let _ = button.style().set_property("opacity", "1");
                    button.set_disabled(false);
                } else {
                    let _ = button.style().set_property("opacity", "0.5");
                    button.set_disabled(true);
                }
            }
        });
    }

    let on_confirm = {
        let on_close = on_close.clone();
        let old_name = old_name.clone();
        let is_project = *is_project;
        Callback::from(move |_| {
            let new_name = new_name.clone();
            let state = state.clone();
            let dispatch = dispatch.clone();
            let old_name = old_name.clone();
            if !*is_data_valid {
                return;
            }
            spawn_local(async move {
                let mut complete_path = PathBuf::from(&state.project.as_ref().unwrap().path);
                if is_project {
                    complete_path.pop();
                } else {
                    complete_path.push("Chapters");
                }
                let args = RenameArgs {
                    path: complete_path.clone(),
                    old: old_name.clone(),
                    new: (*new_name).clone(),
                };
                let args = to_value(&args).unwrap();
                invoke("rename_path", args).await;

                if let Some(mut temp_project) = state.project.clone() {
                    if is_project {
                        temp_project.path = complete_path.join(&(*new_name));
                    } else {
                        for chapter in &mut temp_project.chapters {
                            if *chapter == old_name {
                                chapter.clone_from(&*new_name);
                            }
                        }
                    }
                    dispatch.reduce_mut(|x| x.project = Some(temp_project));
                }
            });
            on_close.emit(MouseEvent::new("Dummy").unwrap());
        })
    };
    let content = html! {
        <>
            <div class="text-xl font-bold">{ format!("Rename \"{}\"", old_name) }</div>
            <br />
            <div class="font-semibold">{ "New Name:" }</div>
            <div
                class="flex rounded-lg border-2 my-2 border-transparent hover:border-primary border-solid"
            >
                <input
                    oninput={new_name_input}
                    ref={new_name_ref}
                    class="w-full bg-crust text-text p-2 rounded-tl-lg rounded-bl-lg border-0 font-standard text-base"
                />
            </div>
            <div id="footer" class="flex justify-end w-full pt-8">
                <div class="text-text underline decoration-primary break-words mr-auto">
                    { (*error_message).clone() }
                </div>
                <button
                    ref={confirm_ref}
                    onclick={on_confirm}
                    class="rounded-lg text-lg px-2 py-1 ml-4 bg-primary text-crust hover:scale-105 border-0"
                >
                    { "Confirm" }
                </button>
                <button
                    onclick={on_close}
                    class="rounded-lg text-lg px-2 py-1 ml-4 bg-secondary text-crust hover:scale-105 border-0"
                >
                    { "Cancel" }
                </button>
            </div>
        </>
    };
    html! { <Modal content={content} /> }
}

fn text_input_handler(value: UseStateHandle<String>) -> Callback<InputEvent> {
    Callback::from(move |e: InputEvent| {
        if let Some(input) = e.target_dyn_into::<HtmlInputElement>() {
            let text = input.value();
            value.set(text);
        }
    })
}
