use std::path::Path;

use serde_wasm_bindgen::to_value;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew::virtual_dom::VNode;
use yew_icons::Icon;
use yew_icons::IconId;
use yewdux::prelude::*;

#[path = "buttons.rs"]
pub mod buttons;
pub use buttons::{ButtonContainer, Props as ButtonProps};

use crate::app::invoke;
use crate::app::wizard::PathArgs;
use crate::app::State;

fn get_file_name(path: &Path) -> String {
    path.to_str()
        .unwrap()
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or_else(|| path.to_str().unwrap())
        .to_string()
}

#[function_component(SideBarWrapper)]
pub fn sidebarwrapper() -> Html {
    let (state, _dispatch) = use_store::<State>();
    if state.project.is_none() {
        html! {}
    } else {
        html! { <SideBar /> }
    }
}

#[function_component(SideBar)]
pub fn sidebar() -> Html {
    let (state, dispatch) = use_store::<State>();
    let title = use_state(|| get_file_name(&(state.project).as_ref().unwrap().path));
    let name_display = use_state(|| html! { (*title).clone() });
    let chapters = use_state(Vec::<VNode>::new);
    let tabs = vec!["Overview".to_string(), "Notes".to_string()];
    let note_types = vec!["Project Notes".to_string(), "Chapter Notes".to_string()];
    let tab = use_state(|| tabs[0].clone());
    let note_tab = use_state(|| note_types[0].clone());

    {
        let title = title.clone();
        let name_display = name_display.clone();
        let state = state.clone();
        use_effect_with(state.clone(), move |_| {
            title.set(get_file_name(&(state.project).as_ref().unwrap().path));
            name_display.set(html! { get_file_name(&(state.project).as_ref().unwrap().path) });
        });
    }

    {
        let chapters = chapters.clone();

        let state = state.clone();
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
        let dispatch = dispatch.clone();
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

    html! {
        <>
            <div
                id="file-explorer"
                class="select-none cursor-default transition h-full overflow-auto flex flex-col"
            >
                <div
                    class="items-center overflow-hidden text-2xl text-subtext hover:text-text my-2 shrink-0"
                >
                    { (*name_display).clone() }
                </div>
                <div class="w-full flex my-2">
                    <div class="rounded-full bg-base py-2 px-4 mr-2 cursor-pointer grow">
                        { "Settings" }
                    </div>
                    <div class="rounded-full bg-base py-2 px-4 mr-2 cursor-pointer grow">
                        { "Statistics" }
                    </div>
                    <div class="rounded-full bg-base py-2 px-4 cursor-pointer grow">
                        { "Extras" }
                    </div>
                </div>
                <TabMenu tabs={tabs} active_tab={tab.clone()} />
                if *tab == "Overview" {
                    <div class="overflow-scroll grow shrink">
                        { for (*chapters).clone() }
                        <div
                            class="w-full hover:bg-base rounded-lg flex justify-center items-center cursor-pointer"
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
                        </div>
                    </div>
                } else {
                    <TabMenu tabs={note_types} active_tab={note_tab.clone()} />
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
    ];

    let tabs = tabs
        .iter()
        .enumerate()
        .map(|(index, tab)| {
            html! {
                <div
                    class={classes!(standard_classes.clone(),
                        if **active_tab == *tab { "bg-primary text-mantle" } else { "bg-base" },
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
                </div>
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
}

#[function_component(ChapterComponent)]
fn chapter(
    ChapterProps {
        chapter,
        index,
        active,
    }: &ChapterProps,
) -> Html {
    let (_state, dispatch) = use_store::<State>();

    let on_load = {
        let index = *index;

        Callback::from(move |_: MouseEvent| {
            let dispatch = dispatch.clone();

            dispatch.reduce_mut(|x| x.project.as_mut().unwrap().active_chapter = Some(index));
        })
    };

    let button_props = vec![
        ButtonProps {
            callback: Callback::from(|_: MouseEvent| {}),
            icon: IconId::LucideEdit3,
            title: "Rename".to_string(),
            size: 1.3,
        },
        ButtonProps {
            callback: Callback::from(|_: MouseEvent| {}),
            icon: IconId::LucideTrash2,
            title: "Delete".to_string(),
            size: 1.3,
        },
    ];

    html! {
        <div
            class={classes!("hover:bg-mantle", "flex", "flex-row", "rounded-lg", "cursor-pointer", "group/buttoncontainer", "pr-3", if *active {"bg-base"} else {""})}
            onclick={on_load}
        >
            <div
                class={classes!("p-2", "w-8", "h-8", "rounded-lg", "text-mantle", "m-2", "justify-center", "items-center", "flex", "text-2xl", if *active {"bg-secondary"} else {"bg-primary"})}
            >
                { *index+1 }
            </div>
            <div class={classes!("flex", "items-center")}>{ chapter.clone() }</div>
            <ButtonContainer button_props={button_props} />
        </div>
    }
}
