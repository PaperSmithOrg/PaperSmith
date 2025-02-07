use std::path::Path;
use std::path::PathBuf;

use serde_wasm_bindgen::to_value;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlTextAreaElement;
use yew::prelude::*;
use yew::virtual_dom::VNode;
use yew_icons::Icon;
use yew_icons::IconId;
use yewdux::prelude::*;

#[path = "buttons.rs"]
pub mod buttons;
pub use buttons::{ButtonContainer, Props as ButtonProps};

#[path = "renaming-modal.rs"]
mod renaming_modal;
use renaming_modal::RenamingModal;

use crate::app::invoke;
use crate::app::modal::Modal;
use crate::app::wizard::PathArgs;
use crate::app::FileWriteData;
use crate::app::State;

fn get_file_name(path: &Path) -> String {
    path.to_str()
        .unwrap()
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or_else(|| path.to_str().unwrap())
        .to_string()
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub modal: UseStateHandle<VNode>,
}

#[function_component(SideBarWrapper)]
pub fn sidebarwrapper(Props { modal }: &Props) -> Html {
    let (state, _dispatch) = use_store::<State>();
    if state.project.is_none() {
        html! {}
    } else {
        html! { <SideBar modal={modal.clone()} /> }
    }
}

#[function_component(SideBar)]
pub fn sidebar(Props { modal }: &Props) -> Html {
    let (state, dispatch) = use_store::<State>();
    let title = use_state(|| get_file_name(&(state.project).as_ref().unwrap().path));
    let chapters = use_state(Vec::<VNode>::new);
    let tabs = vec!["Overview".to_string(), "Notes".to_string()];
    let note_types = vec!["Project".to_string(), "Chapter".to_string()];
    let tab = use_state(|| tabs[0].clone());
    let note_tab = use_state(|| note_types[0].clone());
    let note_ref = use_node_ref();

    {
        let state = state.clone();
        let note_tab = note_tab.clone();
        let note_types = note_types.clone();
        let note_ref = note_ref.clone();
        use_effect_with((tab.clone(), note_tab.clone()), move |_| {
            if let Some(input) = note_ref.cast::<HtmlTextAreaElement>() {
                spawn_local(async move {
                    let mut note_path = state.project.as_ref().unwrap().path.clone();
                    if *note_tab != note_types[0] {
                        note_path.push("Chapters");
                        let project_ref = state.project.as_ref().unwrap();
                        note_path.push(
                            project_ref.chapters[project_ref.active_chapter.unwrap()].clone(),
                        );
                    }
                    note_path.push("Note");
                    note_path.set_extension("md");
                    let content = invoke(
                        "get_file_content",
                        to_value(&PathArgs {
                            path: note_path.to_str().unwrap().to_string(),
                        })
                        .unwrap(),
                    )
                    .await
                    .as_string()
                    .unwrap();
                    input.set_value(&content);
                });
            }
        });
    }

    {
        let title = title.clone();
        let state = state.clone();
        use_effect_with(state.clone(), move |_| {
            title.set(get_file_name(&(state.project).as_ref().unwrap().path));
        });
    }

    {
        let chapters = chapters.clone();

        let state = state.clone();
        let modal = modal.clone();
        use_effect_with(state.clone(), move |_| {
            chapters.set(Vec::new());

            if let Some(project_data) = state.project.as_ref() {
                let new_chapters = project_data
                    .chapters
                    .iter()
                    .enumerate()
                    .map(|(index, chapter)| {
                        html! {
                            <ChapterComponent
                                key={chapter.clone()}
                                chapter={chapter.clone()}
                                index={index}
                                active={project_data.active_chapter == Some(index)}
                                modal={modal.clone()}
                            />
                        }
                    })
                    .collect::<Vec<VNode>>();

                chapters.set(new_chapters);
            }
        });
    }

    let on_add_chapter = {
        let state = state.clone();
        Callback::from(move |_: MouseEvent| {
            let state = state.clone();
            let dispatch = dispatch.clone();
            spawn_local(async move {
                let mut check_path = state.project.as_ref().unwrap().path.clone();
                check_path.push("Chapters");
                check_path.push("Untitled");
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
                    index += 1;
                }
                invoke(
                    "add_chapter",
                    to_value(&PathArgs {
                        path: check_path.to_str().unwrap().to_string(),
                    })
                    .unwrap(),
                )
                .await;
                let mut temp_project = state.project.as_ref().unwrap().clone();
                temp_project
                    .chapters
                    .push(check_path.file_name().unwrap().to_string_lossy().into());
                dispatch.reduce_mut(|state| state.project = Some(temp_project));
            });
        })
    };

    let on_extras = {
        let state = state.clone();
        Callback::from(move |_| {
            let state = state.clone();
            spawn_local(async move {
                let project_clone = state.project.as_ref().unwrap().clone();
                let mut extras_path = project_clone.path.clone();
                extras_path.push("Extras");
                invoke(
                    "open_explorer",
                    to_value(&PathArgs {
                        path: extras_path.to_str().unwrap().to_string(),
                    })
                    .unwrap(),
                )
                .await;
            });
        })
    };

    let on_close = {
        let modal = modal.clone();
        Callback::from(move |_| modal.set(html!()))
    };
    let rename_callback = {
        let title = title.clone();
        let modal = modal.clone();
        Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
            let title = title.clone();
            let on_close = on_close.clone();
            modal.set(html! {
                <RenamingModal
                    old_name={(*title).clone()}
                    closing_callback={on_close}
                    is_project=true
                />
            });
        })
    };

    let note_input_handler = {
        let note_tab = note_tab.clone();
        let note_types = note_types.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(input) = e.target_dyn_into::<HtmlTextAreaElement>() {
                let text = input.value();
                let mut note_path = state.project.as_ref().unwrap().path.clone();
                if *note_tab != note_types[0] {
                    note_path.push("Chapters");
                    let project_ref = state.project.as_ref().unwrap();
                    note_path
                        .push(project_ref.chapters[project_ref.active_chapter.unwrap()].clone());
                }
                note_path.push("Note");
                note_path.set_extension("md");
                spawn_local(async move {
                    let write_data = FileWriteData {
                        path: note_path.to_str().unwrap().to_string(),
                        content: text,
                    };

                    invoke(
                        "write_to_file",
                        serde_wasm_bindgen::to_value(&write_data).unwrap(),
                    )
                    .await;
                });
            }
        })
    };

    html! {
        <>
            <div
                id="file-explorer"
                class="select-none cursor-default transition h-full overflow-auto flex flex-col px-2"
            >
                <button
                    class="items-center overflow-hidden text-2xl text-subtext hover:text-text my-2 shrink-0 cursor-pointer border-0 rounded-full bg-crust text-start"
                    onclick={rename_callback}
                >
                    { (*title).clone() }
                </button>
                <div class="w-full flex my-2 text-center">
                    // <div class="rounded-full bg-base py-2 px-4 mr-2 cursor-pointer grow">
                    //     { "Settings" }
                    // </div>
                    // <div class="rounded-full bg-base py-2 px-4 mr-2 cursor-pointer grow">
                    //     { "Statistics" }
                    // </div>
                    // Space for future buttons
                    <button
                        class="rounded-full bg-base py-2 px-4 cursor-pointer grow border-0 text-inherit text-[length:inherit] hover:bg-mantle"
                        onclick={on_extras}
                    >
                        { "Extras" }
                    </button>
                </div>
                <TabMenu tabs={tabs} active_tab={tab.clone()} />
                if *tab == "Overview" {
                    <div class="overflow-scroll grow shrink p-2">
                        { for (*chapters).clone() }
                        <button
                            class="w-full hover:bg-mantle bg-crust rounded-lg flex justify-center items-center cursor-pointer border-0 text-inherit text-[length:inherit] "
                            onclick={on_add_chapter}
                        >
                            <div class="h-16 flex items-center align-center">
                                <Icon
                                    icon_id={IconId::LucidePlus}
                                    width="2em"
                                    height="2em"
                                    title="Add Chapter"
                                />
                            </div>
                        </button>
                    </div>
                } else {
                    <TabMenu tabs={note_types} active_tab={note_tab.clone()} />
                    <textarea
                        class="h-full overflow-scroll grow shrink bg-base resize-none border-0 focus:ring-0 text-text rounded-lg"
                        oninput={note_input_handler}
                        ref={note_ref}
                    />
                }
            </div>
        </>
    }
}

#[derive(Properties, PartialEq)]
struct TabMenuProps {
    pub tabs: Vec<String>,
    pub active_tab: UseStateHandle<String>,
}

#[function_component(TabMenu)]
fn tabmenu(TabMenuProps { tabs, active_tab }: &TabMenuProps) -> Html {
    let standard_classes = vec![
        "py-2",
        "px-4",
        "cursor-pointer",
        "flex",
        "flex-1",
        "justify-center",
        "border-0",
        "text-inherit",
        "text-[length:inherit]",
    ];

    let tabs = tabs
        .iter()
        .enumerate()
        .map(|(index, tab)| {
            html! {
                <button
                    class={classes!(standard_classes.clone(),
                        if **active_tab == *tab { "bg-primary text-mantle" } else { "bg-base hover:bg-mantle" },
                        if tabs.len() == 1 { "rounded-full"} else {""},
                        if index == 0 { "rounded-l-full" } else {""},
                        if index == tabs.len()-1 { "rounded-r-full" } else {""},
                    )}
                    onclick={Callback::from({
                          let active_tab = active_tab.clone();
                          let tab_clone = tab.clone();
                          move |_| active_tab.set(tab_clone.clone())
                    })}
                >
                    { tab }
                </button>
            }
        })
        .collect::<Vec<VNode>>();

    html! { <div class="w-full flex my-2 justify-around gap-1">{ for tabs }</div> }
}

#[derive(Properties, PartialEq)]
struct ChapterProps {
    pub chapter: String,
    pub index: usize,
    pub active: bool,
    pub modal: UseStateHandle<VNode>,
}

#[function_component(ChapterComponent)]
fn chapter(
    ChapterProps {
        chapter,
        index,
        active,
        modal,
    }: &ChapterProps,
) -> Html {
    let (state, dispatch) = use_store::<State>();

    let on_load = {
        let index = *index;
        let dispatch = dispatch.clone();

        Callback::from(move |_: MouseEvent| {
            let dispatch = dispatch.clone();

            dispatch.reduce_mut(|x| x.project.as_mut().unwrap().active_chapter = Some(index));
        })
    };

    let on_close = {
        let modal = modal.clone();
        Callback::from(move |_| modal.set(html!()))
    };
    let rename_callback = {
        let chapter = chapter.clone();
        let modal = modal.clone();
        let on_close = on_close.clone();
        Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
            let chapter = chapter.clone();
            let on_close = on_close.clone();
            modal.set(html! {
                <RenamingModal old_name={chapter} closing_callback={on_close} is_project=false />
            });
        })
    };
    let delete_callback = {
        let modal = modal.clone();
        let chapter = chapter.clone();
        let on_delete = {
            let on_close = on_close.clone();
            let chapter = chapter.clone();
            Callback::from(move |_: MouseEvent| {
                let state = state.clone();
                let chapter = chapter.clone();
                let dispatch = dispatch.clone();
                spawn_local(async move {
                    let mut complete_path = PathBuf::from(&state.project.as_ref().unwrap().path);
                    complete_path.push("Chapters");
                    complete_path.push(&chapter);
                    let args = PathArgs {
                        path: complete_path.to_str().unwrap().to_string(),
                    };
                    let args = to_value(&args).unwrap();
                    invoke("delete_path", args).await;

                    if let Some(mut temp_project) = state.project.clone() {
                        temp_project.chapters.retain(|x| **x != chapter);
                        dispatch.reduce_mut(|x| x.project = Some(temp_project));
                    }
                });
                on_close.emit(MouseEvent::new("Dummy").unwrap());
            })
        };
        Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
            let on_close = on_close.clone();
            let on_delete = on_delete.clone();
            let content = html! {
                <>
                    <div class="text-xl font-bold">
                        { format!("Do you really want to delete \"{}\"?", chapter) }
                    </div>
                    <br />
                    <div id="footer" class="flex justify-end w-full pt-8">
                        <button
                            onclick={on_delete}
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
            modal.set(html! { <Modal content={content} /> });
        })
    };

    let button_props = vec![
        ButtonProps {
            callback: rename_callback,
            icon: IconId::LucideEdit3,
            title: "Rename".to_string(),
            size: 1.3,
        },
        ButtonProps {
            callback: delete_callback,
            icon: IconId::LucideTrash2,
            title: "Delete".to_string(),
            size: 1.3,
        },
    ];

    html! {
        <button
            class={classes!("hover:bg-mantle", "flex", "flex-row","items-center", "rounded-lg", "cursor-pointer", "group/buttoncontainer", "pr-3", "w-full", "border-0", "text-inherit", "text-[length:inherit]", if *active {"bg-base"} else {"bg-crust"})}
            onclick={on_load}
        >
            <div
                class={classes!("p-2", "w-8", "h-8", "rounded-lg", "text-mantle", "m-2", "justify-center", "items-center", "flex", "text-2xl", if *active {"bg-secondary"} else {"bg-primary"})}
            >
                { *index+1 }
            </div>
            <div class={classes!("flex", "items-center")}>{ chapter.clone() }</div>
            <ButtonContainer button_props={button_props} />
        </button>
    }
}
