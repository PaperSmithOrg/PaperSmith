use std::path::Path;
use std::path::PathBuf;

use serde_wasm_bindgen::to_value;
use web_sys::Element;
use web_sys::HtmlTextAreaElement;
use yew::platform::spawn_local;
use yew::prelude::*;
use yew::virtual_dom::VNode;
use yew_icons::Icon;
use yew_icons::IconId;
use yewdux::prelude::*;

#[path = "renaming-modal.rs"]
mod renaming_modal;
use renaming_modal::RenamingModal;

use crate::app::invoke;
use crate::app::modal_system::Modal;
use crate::app::wizard::PathArgs;
use crate::app::ButtonContainer;
use crate::app::ButtonProps;
use crate::app::FileWriteData;
use crate::app::State;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum ChapterStatus {
    Normal,
    Active,
    ActiveChanges,
}

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
    let open = use_state(|| true);
    let toggle = {
        let open = open.clone();
        Callback::from(move |_| open.set(!*open))
    };
    if state.project.is_none() {
        html! {}
    } else {
        html! {
            <div class="flex h-full w-full flex-row relative">
                if *open {
                    <SideBar modal={modal.clone()} />
                }
                <button
                    class="absolute right-0 w-2 bg-transparent p-0 border-y-0 border-transparent h-full cursor-pointer border-solid hover:border-x-2 hover:border-subtext"
                    ondblclick={toggle}
                />
            </div>
        }
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
                        let mut status = ChapterStatus::Normal;
                        if project_data.active_chapter == Some(index) {
                            if state.changes {
                                status = ChapterStatus::ActiveChanges;
                            } else {
                                status = ChapterStatus::Active;
                            }
                        }
                        html! {
                            <div class="relative">
                                <ChapterComponent
                                    key={chapter.clone()}
                                    chapter={chapter.clone()}
                                    index={index}
                                    status={status}
                                    modal={modal.clone()}
                                />
                                <DragHandler index={index+1} />
                            </div>
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
                class="select-none cursor-default transition h-full overflow-auto flex flex-col px-2 min-w-[18rem]"
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
                        <div class="relative">
                            <DragHandler index=0 />
                        </div>
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
    pub status: ChapterStatus,
    pub modal: UseStateHandle<VNode>,
}

#[function_component(ChapterComponent)]
fn chapter(
    ChapterProps {
        chapter,
        index,
        status,
        modal,
    }: &ChapterProps,
) -> Html {
    let (state, dispatch) = use_store::<State>();

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
        let dispatch = dispatch.clone();
        let state = state.clone();
        let on_close = on_close.clone();
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

    let on_load = {
        let index = *index;

        let dispatch = dispatch.clone();
        Callback::from(move |_: MouseEvent| {
            let dispatch = dispatch.clone();
            dispatch.reduce_mut(|x| {
                x.project.as_mut().unwrap().active_chapter = Some(index);
                x.changes = false;
            });
        })
    };
    let on_load_and_close = {
        let on_load = on_load.clone();
        let on_close = on_close.clone();

        Callback::from(move |_: MouseEvent| {
            on_load.emit(MouseEvent::new("Dummy").unwrap());
            on_close.emit(MouseEvent::new("Dummy").unwrap());
        })
    };
    let on_load_wrapper = {
        let load_modal = html! {
            <>
                <div class="text-xl font-bold">
                    { format!("You have unsaved changes! do you really want to continue?") }
                </div>
                <br />
                <div id="footer" class="flex justify-end w-full pt-8">
                    <button
                        onclick={on_load_and_close.clone()}
                        class="rounded-lg text-lg px-2 py-1 ml-4 bg-primary text-crust hover:scale-105 border-0"
                    >
                        { "Continue" }
                    </button>
                    <button
                        onclick={on_close.clone()}
                        class="rounded-lg text-lg px-2 py-1 ml-4 bg-secondary text-crust hover:scale-105 border-0"
                    >
                        { "Cancel" }
                    </button>
                </div>
            </>
        };
        let modal = modal.clone();
        let state = state.clone();
        Callback::from(move |_: MouseEvent| {
            let load_modal = load_modal.clone();
            let state = state.clone();
            let modal = modal.clone();
            let on_load = on_load.clone();
            if state.changes {
                modal.set(html! { <Modal content={load_modal} /> });
            } else {
                on_load.emit(MouseEvent::new("Dummy").unwrap());
            }
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

    let ondragstart = {
        let index = *index;
        let state = state.clone();
        let dispatch = dispatch.clone();
        Callback::from(move |e: DragEvent| {
            let data_transfer = e.data_transfer().unwrap();
            data_transfer.set_drag_image(&e.target_dyn_into::<Element>().unwrap(), 0, 0);
            let _ = data_transfer.set_data("text", &index.to_string());

            gloo_console::log!(format!("Drag Start: {:?}", state.dragger));
            dispatch.reduce_mut(|x| x.dragger = Some(index));
        })
    };

    let ondragend = {
        Callback::from(move |_e: DragEvent| {
            gloo_console::log!(format!("Drag End: {:?}", state.dragger));
            dispatch.reduce_mut(|x| x.dragger = None);
        })
    };

    html! {
        <button
            class={classes!("hover:bg-mantle", "flex", "flex-row","items-center", "rounded-lg", "cursor-pointer", "group/buttoncontainer","p-0", "pr-3", "w-full", "border-0", "text-inherit", "text-[length:inherit]",
                if *status==ChapterStatus::Active || *status==ChapterStatus::ActiveChanges {"bg-base"} else {"bg-crust"},
                if *status==ChapterStatus::ActiveChanges {"italic"} else {""}
            )}
            draggable="true"
            onclick={on_load_wrapper}
            ondragstart={ondragstart}
            ondragend={ondragend}
        >
            <div
                class={classes!("p-2", "w-8", "h-8", "rounded-lg", "text-mantle", "m-2", "justify-center", "items-center", "flex", "text-2xl",
                    if *status==ChapterStatus::Active || *status==ChapterStatus::ActiveChanges {"bg-secondary"} else {"bg-primary"})}
            >
                { *index+1 }
            </div>
            <div class={classes!("flex", "items-center")}>{ chapter.clone() }</div>
            <ButtonContainer button_props={button_props} />
        </button>
    }
}

#[derive(Properties, PartialEq)]
struct DragHandlerProps {
    // The index of where the chapter will be moved when the handler triggers
    pub index: usize,
}

#[function_component(DragHandler)]
fn draghandler(DragHandlerProps { index }: &DragHandlerProps) -> Html {
    let (state, dispatch) = use_store::<State>();
    let active = use_state(|| false);

    // Only show if dragged chapter is inside the handler
    let ondragover = {
        let active = active.clone();
        Callback::from(move |e: DragEvent| {
            e.prevent_default();
            active.set(true);
        })
    };
    let ondragout = {
        let active = active.clone();
        Callback::from(move |e: DragEvent| {
            e.prevent_default();
            active.set(false);
        })
    };

    let ondrop = {
        let index = *index;
        let active = active.clone();
        Callback::from(move |e: DragEvent| {
            e.prevent_default();
            dispatch.reduce_mut(|x| {
                // Get references to the state
                let (Some(project), Some(dragger_index)) = (x.project.as_mut(), x.dragger.as_ref())
                else {
                    return;
                };
                let Some(dragger) = project.chapters.get(*dragger_index).cloned() else {
                    return;
                };

                // Adjust index if it would be moved my removing an the chapter
                let adjusted_index = if *dragger_index < index {
                    index - 1
                } else {
                    index
                };

                // Move the chapter
                project.chapters.remove(*dragger_index);
                project.chapters.insert(adjusted_index, dragger);

                // Adjust active chapter if it's been moved
                let Some(active_chapter) = project.active_chapter else {
                    return;
                };
                if active_chapter == adjusted_index {
                    project.active_chapter = Some(*dragger_index);
                }
                if active_chapter == *dragger_index {
                    project.active_chapter = Some(adjusted_index);
                }

                // Clean up drag handling
                x.dragger = None;
                active.set(false);
            });
        })
    };

    html! {
        if state.dragger.is_some() {
            <div
                class="absolute -bottom-4 h-6 w-full z-30"
                ondragover={ondragover}
                ondragleave={ondragout}
                ondrop={ondrop}
            >
                if *active {
                    <div
                        class="absolute top-1/2 transform -translate-y-1/2 w-full border-b-2 border-solid border-primary pointer-events-none"
                    />
                }
            </div>
        }
    }
}
