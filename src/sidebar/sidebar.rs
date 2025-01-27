use std::path::Path;

use buttons::Button;
use deleting::get_delete_callback;
use serde_wasm_bindgen::to_value;
use shared::Chapter;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlElement;
use yew::prelude::*;
use yew::virtual_dom::VNode;
use yew_icons::IconId;
use yewdux::prelude::*;

#[path = "renaming.rs"]
mod renaming;
use renaming::get_rename_callback;
use renaming::RenameKind;

#[path = "deleting.rs"]
mod deleting;

#[path = "dropdown.rs"]
mod dropdown;

use dropdown::Dropdown;
use dropdown::Type;

#[path = "buttons.rs"]
pub mod buttons;
pub use buttons::{ButtonContainer, Props as ButtonProps};

use crate::app::invoke;
use crate::app::wizard::PathArgs;
use crate::app::State;

#[derive(Properties, PartialEq)]
pub struct SideBarProps {
    pub input_ref: NodeRef,
}

fn get_file_name(path: &Path) -> String {
    path.to_str()
        .unwrap()
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or_else(|| path.to_str().unwrap())
        .to_string()
}

#[function_component(SideBarWrapper)]
pub fn sidebarwrapper(props: &SideBarProps) -> Html {
    let (state, _dispatch) = use_store::<State>();
    if state.project.is_none() {
        html! {}
    } else {
        html! { <SideBar input_ref={props.input_ref.clone()} /> }
    }
}
#[function_component(SideBar)]
pub fn sidebar(SideBarProps { input_ref }: &SideBarProps) -> Html {
    let (state, dispatch) = use_store::<State>();
    let rename_input_ref = use_node_ref();
    let title = use_state(|| get_file_name(&(state.project).as_ref().unwrap().path));
    let name_display = use_state(|| html! { (*title).clone() });
    let chapters = use_state(Vec::<VNode>::new);

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
        let input_ref = input_ref.clone();

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
                                key={chapter.name.clone()}
                                chapter={chapter.clone()}
                                index={index}
                                input_ref={input_ref.clone()}
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
        Callback::from(move |_| {
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
                temp_project.chapters.push(Chapter {
                    name: check_path.file_name().unwrap().to_string_lossy().into(),
                    notes: Vec::new(),
                    extras: Vec::new(),
                });
                dispatch.set(State {
                    project: Some(temp_project),
                });
            });
        })
    };

    let button_props = [
        ButtonProps {
            callback: get_rename_callback(
                name_display.clone(),
                title,
                rename_input_ref.clone(),
                state,
                dispatch,
                RenameKind::Book,
                None,
            ),
            icon: IconId::LucideEdit3,
            title: "Rename Book".to_string(),
            size: 1.,
        },
        ButtonProps {
            callback: on_add_chapter,
            icon: IconId::LucidePlus,
            title: "Create Chapter".to_string(),
            size: 1.,
        },
    ];

    html! {
        <>
            <div id="file-explorer" class="select-none outline-none">
                <div
                    class="group/buttoncontainer items-center flex relative transition text-ellipsis whitespace-nowrap overflow-hidden cursor-default text-xl"
                >
                    <div ref={rename_input_ref.clone()} class="pl-2 mb-1">
                        { (*name_display).clone() }
                    </div>
                    <div class="flex items-center ml-auto my-auto">
                        { button_props
                        .iter()
                        .map(|props| {
                            html! { <>
                                <Button callback={props.callback.clone()} icon={props.icon} size={props.size} title={props.title.clone()}/>
                            </>
                            }
                        })
                        .collect::<Html>() }
                    </div>
                </div>
                <div
                    class="text-lg border-l-2 border-r-0 border-y-0 border-solid border-text pl-2 ml-2"
                >
                    { for (*chapters).clone() }
                </div>
            </div>
        </>
    }
}

#[derive(Properties, PartialEq)]
struct ChapterProps {
    pub chapter: Chapter,
    pub index: usize,
    pub input_ref: NodeRef,
    pub active: bool,
}

#[function_component(ChapterComponent)]
fn chapter(
    ChapterProps {
        chapter,
        index,
        input_ref,
        active,
    }: &ChapterProps,
) -> Html {
    let (state, dispatch) = use_store::<State>();
    let note_elements: Vec<Html> = chapter
        .notes
        .iter()
        .map(|note| {
            html! { <Entry key={note.clone()} name={note.clone()} chapter_index={index} /> }
        })
        .collect();
    let on_extras = {
        let index = *index;
        let state = state.clone();
        Callback::from(move |_| {
            let state = state.clone();
            spawn_local(async move {
                let project_clone = state.project.as_ref().unwrap().clone();
                let mut extras_path = project_clone.path.clone();
                extras_path.push("Chapters");
                extras_path.push(project_clone.chapters[index].name.clone());
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

    let mut content_path = (state.project).as_ref().unwrap().path.clone();
    content_path.push("Chapters");
    content_path.push(chapter.name.clone());
    content_path.push("Content");
    content_path.set_extension("md");
    let on_load = {
        let input_ref = input_ref.clone();
        let chapter = chapter.clone();

        Callback::from(move |_| {
            let dispatch = dispatch.clone();
            let state = state.clone();
            let content_path = content_path.clone();
            let input_ref = input_ref.clone();

            let index = state
                .project
                .clone()
                .unwrap()
                .chapters
                .iter()
                .position(|r| r.name == chapter.name)
                .unwrap();

            dispatch.reduce_mut(|x| x.project.as_mut().unwrap().active_chapter = Some(index));

            spawn_local(async move {
                let content_path = content_path.clone();
                let content = invoke(
                    "get_file_content",
                    to_value(&PathArgs {
                        path: content_path.to_str().unwrap().to_string(),
                    })
                    .unwrap(),
                )
                .await
                .as_string()
                .unwrap();

                if let Some(input_element) = input_ref.cast::<HtmlElement>() {
                    input_element.set_inner_text(content.as_str());
                    let _ = input_element.dispatch_event(&InputEvent::new("input").unwrap());
                }
            });
        })
    };

    html! {
        <Dropdown
            title={chapter.name.clone()}
            open=false
            dropdown_type={Type::Chapter}
            accent={active}
        >
            <Title onclick={on_load}>
                <div class="pl-5">{ "Contents" }</div>
            </Title>
            //<Dropdown title="Notes" open=false dropdown_type={Type::Notes} chapter_index={index}>
            //    { for note_elements }
            //</Dropdown>
            <Title onclick={on_extras}>
                <div class="pl-5">{ "Extras" }</div>
            </Title>
        </Dropdown>
    }
}

#[derive(Properties, PartialEq, Eq)]
pub struct EntryProps {
    pub name: String,
    pub chapter_index: usize,
}

#[function_component(Entry)]
fn entry(
    EntryProps {
        name,
        chapter_index,
    }: &EntryProps,
) -> Html {
    let (state, dispatch) = use_store::<State>();
    let input_ref = use_node_ref();
    let title = use_state(|| name.clone());
    let name_display = use_state(|| html! { name.clone() });

    let button_props = vec![
        ButtonProps {
            callback: get_rename_callback(
                name_display.clone(),
                title,
                input_ref,
                state.clone(),
                dispatch.clone(),
                RenameKind::Note,
                Some(*chapter_index),
            ),
            icon: IconId::LucideEdit3,
            title: "Rename Note".to_string(),
            size: 1.,
        },
        ButtonProps {
            callback: get_delete_callback(
                state,
                dispatch,
                name.clone(),
                Some(*chapter_index),
                RenameKind::Note,
            ),
            icon: IconId::LucideTrash2,
            title: "Delete Note".to_string(),
            size: 1.,
        },
    ];
    html! {
        <Title button_props={button_props}>
            <div class="pl-2">{ (*name_display).clone() }</div>
        </Title>
    }
}

#[derive(Properties, PartialEq)]
pub struct TitleProps {
    pub children: Html,
    #[prop_or_default]
    pub button_props: Vec<ButtonProps>,
    #[prop_or_default]
    pub onclick: Callback<MouseEvent>,
}

#[function_component(Title)]
fn title(
    TitleProps {
        children,
        button_props,
        onclick,
    }: &TitleProps,
) -> Html {
    html! {
        <div
            class="group/buttoncontainer items-center flex relative transition rounded-md my-[1px] hover:bg-secondary hover:text-mantle cursor-pointer"
            onclick={onclick}
        >
            { children.clone() }
            <ButtonContainer button_props={(*button_props).clone()} />
        </div>
    }
}
//rounded-md hover:bg-green pl-2 hover:text-mantle">
//text-text text-ellipsis whitespace-nowrap overflow-hidden cursor-default text-xl"
//rounded-md my-[1px] pl-5 hover:bg-sapphire hover:text-mantle"
//rounded-md my-[1px] content-center transition text-subtext1 hover:bg-mauve hover:text-mantle"
//rounded-md my-[1px] pl-5 hover:bg-sapphire hover:text-mantle"
