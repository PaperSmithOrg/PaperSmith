use gloo_timers::callback::Timeout;
use serde_wasm_bindgen::to_value;
use yew::platform::spawn_local;
use web_sys::HtmlElement;
use yew::prelude::*;
use yew_icons::IconId;
use yewdux::prelude::*;

#[path = "chevron.rs"]
mod chevron;
use chevron::Chevron;

use crate::app::{
    invoke,
    sidebar::{
        buttons::Props as ButtonProps,
        deleting::get_delete_callback,
        renaming::{get_rename_callback, RenameKind},
        Title,
    },
    wizard::PathArgs,
    State,
};

#[derive(Properties, PartialEq)]
pub struct Props {
    pub title: String,
    pub open: bool,
    pub children: Html,
    pub dropdown_type: Type,
    #[prop_or_default]
    pub accent: bool,
    #[prop_or(None)]
    pub chapter_index: Option<usize>,
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum Type {
    Chapter,
    Notes,
}

#[function_component(Dropdown)]
pub fn dropdown(
    Props {
        title: origininal_title,
        open,
        dropdown_type,
        accent,
        children,
        chapter_index,
    }: &Props,
) -> Html {
    let (state, dispatch) = use_store::<State>();
    let transition_string = use_state(|| "max-height: 0px".to_string());
    let content_ref = use_node_ref();
    let chevron2_hidden = use_state(|| true);
    let chevron_rotated = use_state(|| *open);
    let name_display = use_state(|| html!(origininal_title));
    let input_ref = use_node_ref();
    let title = use_state(|| origininal_title.clone());

    let onclick = {
        let transition_string = transition_string.clone();
        let content_ref = content_ref.clone();
        let chevron2_hidden = chevron2_hidden;
        let chevron1_rotated = chevron_rotated.clone();

        Callback::from(move |_: MouseEvent| {
            if transition_string.contains(" 0px") {
                let content = content_ref
                    .cast::<HtmlElement>()
                    .expect("div_ref not attached to div element");

                transition_string.set(format!(
                    "max-height: {}px; transition: max-height 100ms ease-in-out",
                    content.scroll_height()
                ));

                chevron2_hidden.set(false);
                chevron1_rotated.set(true);

                let _timeout = Timeout::new(100, {
                    let transition_string = transition_string.clone();
                    move || {
                        transition_string.set("max-height: none".to_string());
                    }
                })
                .forget();
            } else {
                let content = content_ref
                    .cast::<HtmlElement>()
                    .expect("div_ref not attached to div element");

                transition_string.set(format!("max-height: {}px", content.scroll_height()));

                let _timeout = Timeout::new(1, {
                    let transition_string = transition_string.clone();
                    move || {
                        transition_string.set(
                            "max-height: 0px; transition: max-height 100ms ease-in-out".to_string(),
                        );
                    }
                })
                .forget();

                chevron2_hidden.set(true);
                chevron1_rotated.set(false);
            }
        })
    };

    let on_rename;
    let on_delete;
    let on_add_note;
    if let Some(unwrapped_project) = state.project.clone() {
        match dropdown_type {
            Type::Chapter => {
                on_rename = get_rename_callback(
                    name_display.clone(),
                    title.clone(),
                    input_ref,
                    state.clone(),
                    dispatch.clone(),
                    RenameKind::Chapter,
                    None,
                );
                on_delete = get_delete_callback(
                    state,
                    dispatch,
                    (*title).clone(),
                    None,
                    RenameKind::Chapter,
                );
                on_add_note = Callback::from(move |_: MouseEvent| {});
            }
            Type::Notes => {
                on_rename = Callback::from(move |_: MouseEvent| {});
                on_delete = Callback::from(move |_: MouseEvent| {});
                on_add_note = {
                    let chapter_index = *chapter_index;
                    let chevron_rotated = chevron_rotated.clone();
                    let onclick = onclick.clone();
                    Callback::from(move |e: MouseEvent| {
                        e.stop_propagation();
                        let unwrapped_project = unwrapped_project.clone();
                        let dispatch = dispatch.clone();
                        let chevron_rotated = chevron_rotated.clone();
                        let onclick = onclick.clone();
                        spawn_local(async move {
                            let mut check_path = unwrapped_project.path.clone();
                            check_path.push("Chapters");
                            check_path.push(
                                unwrapped_project.chapters[chapter_index.unwrap()]
                                    .name
                                    .clone(),
                            );
                            check_path.push("Notes");
                            check_path.push("Untitled");
                            check_path.set_extension("md");
                            let mut index = 1;
                            while !invoke(
                                "can_create_path",
                                to_value(&PathArgs {
                                    path: check_path.to_str().unwrap().to_string().clone(),
                                })
                                .unwrap(),
                            )
                            .await
                            .as_string()
                            .unwrap()
                            .is_empty()
                            {
                                check_path.pop();
                                check_path.push("Untitled".to_string() + &index.to_string());
                                check_path.set_extension("md");
                                index += 1;
                            }
                            invoke(
                                "create_empty_file",
                                to_value(&PathArgs {
                                    path: check_path.to_str().unwrap().to_string(),
                                })
                                .unwrap(),
                            )
                            .await;
                            check_path.set_extension("");
                            let mut temp_project = unwrapped_project.clone();
                            temp_project.chapters[chapter_index.unwrap()]
                                .notes
                                .push(check_path.file_name().unwrap().to_string_lossy().into());
                            dispatch.reduce_mut(|state| state.project = Some(temp_project));
                            if !*chevron_rotated {
                                onclick.emit(e);
                            }
                        });
                    })
                }
            }
        }
    } else {
        on_rename = Callback::from(move |_: MouseEvent| {});
        on_delete = Callback::from(move |_: MouseEvent| {});
        on_add_note = Callback::from(move |_: MouseEvent| {});
    }

    html! {
        <div>
            <Title
                button_props={get_buttons(*dropdown_type, on_rename, on_delete, on_add_note)}
                onclick={onclick}
            >
                <Chevron rotated={*chevron_rotated} hidden=false />
                if *accent {
                    <div
                        class="flex flex-row items-center pl-5 text-ellipsis whitespace-nowrap overflow-hidden"
                    >
                        <div
                            class="w-2 h-2 mr-2 bg-text group-hover/buttoncontainer:bg-mantle rounded-full transition"
                        />
                        { (*name_display).clone() }
                    </div>
                } else {
                    <div
                        class="flex flex-row items-center pl-5 text-ellipsis whitespace-nowrap overflow-hidden"
                    >
                        <div class="w-2 h-2 mr-2 bg-transparent rounded-full" />
                        { (*name_display).clone() }
                    </div>
                }
            </Title>
            <div
                class="pl-2 ml-2 border-solid border-l-2 border-r-0 border-y-0 border-text text-subtext2 overflow-hidden cursor-default"
                style={(*transition_string).clone()}
                ref={content_ref}
            >
                { children.clone() }
            </div>
        </div>
    }
}

fn get_buttons(
    dropdown_type: Type,
    on_rename: Callback<MouseEvent>,
    on_delete: Callback<MouseEvent>,
    on_add_note: Callback<MouseEvent>,
) -> Vec<ButtonProps> {
    match dropdown_type {
        Type::Chapter => {
            vec![
                ButtonProps {
                    callback: on_rename,
                    icon: IconId::LucideEdit3,
                    title: "Rename Chapter".to_string(),
                    size: 1.,
                },
                ButtonProps {
                    callback: on_delete,
                    icon: IconId::LucideTrash2,
                    title: "Delete Chapter".to_string(),
                    size: 1.,
                },
            ]
        }
        Type::Notes => {
            vec![ButtonProps {
                callback: on_add_note,
                icon: IconId::LucidePlus,
                title: "Create Note".to_string(),
                size: 1.,
            }]
        }
    }
}
