use gloo::utils::document;
use gloo_timers::callback::Timeout;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::HtmlDocument;
use web_sys::HtmlElement;
use yew::events::MouseEvent;
use yew::platform::spawn_local;
use yew::prelude::*;
use yew_icons::IconId;
use yewdux::prelude::*;

use shared::Project;
use shared::Settings;

mod notepad;
use notepad::Notepads;

mod styling;
use styling::TextStylingControls;

mod statistic;
use statistic::StatisticWindow;
use statistic::Statistics;

mod sidebar;
use sidebar::SideBarWrapper;

mod buttons;
pub use buttons::{Props as ButtonProps, *};

mod wizard;
use wizard::ProjectWizard;

mod modal_system;
use modal_system::Modal;
use modal_system::VerticalModal;

mod settings;
use settings::SettingsMenu;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "tauri"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[derive(Properties, PartialEq)]
pub struct WordCountProps {
    pub pages_ref: NodeRef,
}

#[derive(Serialize)]
pub struct PathArgs {
    pub path: String,
}

#[derive(Serialize, Deserialize)]
pub struct FileWriteData {
    pub path: String,
    pub content: String,
}

#[derive(Default, Clone, PartialEq, Eq, Store, Debug)]
pub struct State {
    project: Option<Project>,
    settings: Option<Settings>,
    changes: bool,
    dragger: Option<usize>,
}

#[derive(Serialize, Deserialize)]
pub struct ProjectProps {
    project: Project,
}

#[function_component(App)]
pub fn app() -> Html {
    let (state, dispatch) = use_store::<State>();
    let modal = use_state(|| html!());

    let text_input_ref = use_node_ref();
    let pages_ref = use_node_ref();

    let save_fn = {
        let text_input_ref = text_input_ref.clone();
        let state = state.clone();

        Callback::from(move |()| {
            let text_input_ref = text_input_ref.clone();
            let state = state.clone();

            spawn_local(async move {
                if state.project.is_none() {
                    return;
                }
                if state.project.as_ref().unwrap().active_chapter.is_none() {
                    return;
                }
                if let Some(input_element) = text_input_ref.cast::<HtmlElement>() {
                    let text = input_element.inner_text();

                    let Some(project) = state.project.as_ref() else {
                        return;
                    };
                    let Some(active_chapter) = project.active_chapter else {
                        return;
                    };
                    let Some(chapter_name) = project.chapters.get(active_chapter) else {
                        return;
                    };

                    let mut path = project.path.clone();
                    path.push("Chapters");
                    path.push(chapter_name);
                    path.push("Content.md");

                    let write_data = FileWriteData {
                        path: path.to_string_lossy().to_string(),
                        content: text,
                    };

                    invoke(
                        "write_to_file",
                        serde_wasm_bindgen::to_value(&write_data).unwrap(),
                    )
                    .await;

                    // TODO: add save notifier
                }
            });
        })
    };

    let save = {
        let dispatch = dispatch.clone();
        Callback::from(move |_: MouseEvent| {
            save_fn.emit(());
            dispatch.reduce_mut(|x| x.changes = false);
        })
    };

    let open_statistics = {
        let modal = modal.clone();
        Callback::from(move |_| {
            let statistic_window = html! {
                <StatisticWindow
                    closing_callback={let modal = modal.clone();
                        Callback::from(move |_| modal.set(html!()))}
                />
            };
            modal.set(html! { <VerticalModal content={statistic_window} /> });
        })
    };

    let open_settings: Callback<MouseEvent> = {
        let modal = modal.clone();
        Callback::from(move |_| {
            modal.set(html! {
                <Modal
                    content={html! {
                    <SettingsMenu
                        closing_callback={
                            let modal = modal.clone();
                            Callback::from(move |_| modal.set(html!()))
                        }
                    />
                    }}
                />
            });
        })
    };

    let on_load = {
        let modal = modal.clone();
        Callback::from(move |_| {
            let modal = modal.clone();
            let dispatch = dispatch.clone();
            spawn_local(async move {
                let project_jsvalue = invoke("get_project", JsValue::null()).await;
                let project_or_none: Option<Project> =
                    serde_wasm_bindgen::from_value(project_jsvalue).unwrap();
                if project_or_none.is_some() {
                    dispatch.reduce_mut(|state| state.project = project_or_none);
                    modal.set(html!());
                }
            });
        })
    };

    let on_undo = Callback::from(move |_| {
        let html_doc: HtmlDocument = document().dyn_into().unwrap();
        let _ = html_doc.exec_command("undo");
    });

    let on_redo = Callback::from(move |_| {
        let html_doc: HtmlDocument = document().dyn_into().unwrap();
        let _ = html_doc.exec_command("redo");
    });

    {
        use_effect_with((), move |()| {
            apply_settings();
        });
    }
    {
        let on_load = on_load.clone();
        let modal = modal.clone();
        let text_input_ref = text_input_ref.clone();
        use_effect_with(state.project.clone(), move |project| {
            if let Some(project) = project.clone() {
                let project_clone = project.clone();
                spawn_local(async move {
                    let _project_jsvalue = invoke(
                        "write_project_config",
                        serde_wasm_bindgen::to_value(&ProjectProps {
                            project: project.clone(),
                        })
                        .unwrap(),
                    )
                    .await;
                });
                if let Some(active_chapter) = project_clone.active_chapter {
                    let project = project_clone.clone();
                    spawn_local(async move {
                        let mut content_path = project.path.clone();
                        content_path.push("Chapters");
                        content_path
                            .push(project_clone.chapters.get(active_chapter).unwrap().clone());
                        content_path.push("Content");
                        content_path.set_extension("md");
                        let content = invoke(
                            "get_file_content",
                            serde_wasm_bindgen::to_value(&PathArgs {
                                path: content_path.to_str().unwrap().to_string(),
                            })
                            .unwrap(),
                        )
                        .await
                        .as_string()
                        .unwrap();

                        if let Some(input_element) = text_input_ref.cast::<HtmlElement>() {
                            input_element.set_inner_text(content.as_str());

                            let _result =
                                input_element.dispatch_event(&InputEvent::new("input").unwrap());
                        }
                    });
                }
                return;
            }
            let blocker = html! {
                <div class="flex flex-col h-32 w-full justify-between">
                    <div class="text-3xl text-center">{ "Please select or create a project" }</div>
                    <div class="flex justify-evenly w-full text-xl">
                        <button
                            class="bg-primary text-mantle p-2 rounded-lg cursor-pointer border-0 text-inherit text-[length:inherit] hover:ring-1 hover:ring-primary"
                            onclick={let on_load = on_load.clone();
                        Callback::from(move |e: MouseEvent| {
                            on_load.emit(e);
                        })}
                        >
                            { "Open Project" }
                        </button>
                        <button
                            class="bg-secondary text-mantle p-2 rounded-lg cursor-pointer border-0 text-inherit text-[length:inherit] hover:ring-1 hover:ring-secondary"
                            onclick={get_wizard_opener(&modal, false)}
                        >
                            { "Create Project" }
                        </button>
                    </div>
                </div>
            };
            modal.set(html! { <Modal content={blocker} /> });
        });
    }

    html! {
        <div class="h-screen w-screen flex flex-col">
            <div class="light lightdark medium dark verydark" />
            <div class="modal-wrapper">{ (*modal).clone() }</div>
            <style id="dynamic-style" />
            <div class="h-8 flex justify-left items-center p-2 bg-crust">
                <Button
                    callback={get_wizard_opener(&modal, true)}
                    icon={IconId::LucideFilePlus}
                    title="Create Project"
                    size=1.5
                />
                <Button
                    callback={on_load}
                    icon={IconId::LucideFolderOpen}
                    title="Load Project"
                    size=1.5
                />
                <Button callback={save} icon={IconId::LucideSave} title="Save" size=1.5 />
                <Button
                    callback={open_settings}
                    icon={IconId::LucideSettings}
                    title="Open Settings"
                    size=1.5
                />
                <div class="w-[1px] h-[20px] bg-subtext my-0 mx-1 " />
                <Button callback={on_undo} icon={IconId::LucideUndo} title="Undo" size=1.5 />
                <Button callback={on_redo} icon={IconId::LucideRedo} title="Redo" size=1.5 />
                <div class="w-[1px] h-[20px] bg-subtext my-0 mx-1 " />
                <TextStylingControls />
            </div>
            <div id="main_content" class="flex flex-1 grow min-h-0 m-3">
                <div class="h-full bg-crust">
                    { html!{<SideBarWrapper modal={modal.clone()}/>} }
                </div>
                <Notepads pages_ref={pages_ref.clone()} text_input_ref={text_input_ref} />
            </div>
            <div
                class="h-3 justify-between items-center flex p-2 bg-crust border-solid border-t-[2px] border-x-0 border-b-0 border-text"
            >
                <div class="bottombar-left">
                    <Statistics pages_ref={pages_ref.clone()} />
                </div>
                <div class="bottombar-right">
                    <Button
                        callback={open_statistics}
                        icon={IconId::LucideBarChart3}
                        title="Statistics"
                        size=1.5
                    />
                </div>
            </div>
        </div>
    }
}

fn apply_settings() {
    spawn_local(async move {
        let path_jsvalue = invoke("get_data_dir", JsValue::NULL).await;

        let mut path = path_jsvalue.as_string().expect("Cast failed").clone();

        path.push_str("/PaperSmith");

        if let Ok(args) = serde_wasm_bindgen::to_value(&PathArgs { path: path.clone() }) {
            let result = invoke("can_create_path", args.clone()).await.as_string();
            if result.unwrap_or_default().is_empty() {
                invoke("create_directory", args).await;
            }
        }

        let mut statistics_path = path.clone();
        statistics_path.push_str("/Statistics");

        if let Ok(args) = serde_wasm_bindgen::to_value(&PathArgs {
            path: statistics_path.clone(),
        }) {
            let result = invoke("can_create_path", args.clone()).await.as_string();
            if result.unwrap_or_default().is_empty() {
                invoke("create_directory", args).await;
            }
        }

        if let Ok(args) = serde_wasm_bindgen::to_value(&PathArgs { path }) {
            let settings_jsvalue = invoke("get_settings", args).await;

            let Ok(settings) = serde_wasm_bindgen::from_value::<Settings>(settings_jsvalue) else {
                return;
            };

            println!("{:?}", settings.theme);

            let _ = Timeout::new(10, move || switch_theme(&settings.theme)).forget();
        }
    });
}

fn switch_theme(theme: &str) {
    let html_doc: HtmlDocument = document().dyn_into().unwrap();
    let body = html_doc.body().unwrap();
    let theme2 = theme.to_lowercase().replace(' ', "");
    body.set_class_name(format!("{theme2} bg-crust text-text").as_str());
}

fn get_wizard_opener(modal: &UseStateHandle<Html>, closable: bool) -> Callback<MouseEvent> {
    let modal = modal.clone();
    Callback::from(move |_| {
        let wizard = html! {
            <ProjectWizard
                closing_callback={let modal = modal.clone();
                    Callback::from(move |_| modal.set(html!()))}
                closable={closable}
            />
        };
        modal.set(html! { <Modal content={wizard} /> });
    })
}
