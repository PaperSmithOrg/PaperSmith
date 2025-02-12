use gloo::utils::document;
use gloo_timers::callback::Timeout;
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::to_value;
use shared::Project;
use sidebar::buttons::Button;
use statistic::StatisticWindow;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::HtmlDocument;
use web_sys::HtmlElement;
use yew::events::MouseEvent;
use yew::platform::spawn_local;
use yew::prelude::*;
use yew_hooks::use_interval;
use yew_icons::IconId;
use yewdux::prelude::*;

use shared::Settings;

#[path = "notepad/notepad.rs"]
mod notepad;
use notepad::Notepads;

#[path = "toolbar/toolbar.rs"]
mod toolbar;
use toolbar::Toolbar;

//#[path = "text_alignment_handlers.rs"]
//mod text_alignment_handlers;
//use text_alignment_handlers::TextAlignmentControls;

#[path = "menubar/text/text_styling_handlers.rs"]
mod text_styling_handlers;
use text_styling_handlers::TextStylingControls;

#[path = "statistics/statistic.rs"]
mod statistic;
use statistic::Statistics;

#[path = "sidebar/sidebar.rs"]
mod sidebar;
use sidebar::SideBarWrapper;

#[path = "project-wizard/wizard.rs"]
mod wizard;
use wizard::ProjectWizard;

#[path = "modal-system/modal.rs"]
mod modal;
use modal::Modal;
use modal::VerticalModal;

#[path = "settings-menu/settings.rs"]
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
}

#[derive(Serialize, Deserialize)]
pub struct ProjectProps {
    project: Project,
}

#[function_component(App)]
pub fn app() -> Html {
    let (state, dispatch) = use_store::<State>();
    let modal = use_state(|| html!());

    let project_path = state.project.as_ref().map(|proj| proj.path.clone());
    let text_input_ref = use_node_ref();
    let pages_ref = use_node_ref();

    let save_fn = {
        let text_input_ref = text_input_ref.clone();
        let state = state.clone();

        Callback::from(move |()| {
            let text_input_ref = text_input_ref.clone();
            let project_path = project_path.clone();
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

                    if let Some(mut path) = project_path {
                        path.push("Chapters");
                        path.push(
                            state
                                .project
                                .as_ref()
                                .unwrap()
                                .chapters
                                .get(state.project.as_ref().unwrap().active_chapter.unwrap())
                                .unwrap(),
                        );
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

                        // modal.set(html! {});
                    }
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
    {
        let save = save.clone();
        use_interval(
            move || {
                save.emit(MouseEvent::new("Dummy").unwrap());
            },
            // if let Some(settings) = state.settings.as_ref() {
            //     settings.interval
            // } else {
            //     Settings::default().interval
            // },
            5 * 60 * 1000,
        ); // 300,000 ms = 5 minutes
    }

    let open_modal = {
        let modal = modal.clone();
        Callback::from(move |_| {
            modal.set(html! {
                <Modal
                    content={html! {
                        <ProjectWizard
                            closing_callback={
                                let modal = modal.clone();
                                Callback::from(move |_| modal.set(html!()))
                            }
                            closable={true}
                        />
                    }}
                />
            });
        })
    };
    let open_modal2 = {
        let modal = modal.clone();
        Callback::from(move |_: MouseEvent| {
            modal.set(html! {
                <Modal
                    content={html! {
                        <ProjectWizard
                            closing_callback={
                                let modal = modal.clone();
                                Callback::from(move |_| modal.set(html!()))
                            }

                    closable={false}
                        />
                    }}
                />
            });
        })
    };

    let open_statistics = {
        let modal = modal.clone();
        Callback::from(move |_| {
            modal.set(html! {
                <VerticalModal
                    content={html! {
                        <StatisticWindow
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
        html_doc.exec_command("undo").unwrap();
    });

    let on_redo = Callback::from(move |_| {
        let html_doc: HtmlDocument = document().dyn_into().unwrap();
        html_doc.exec_command("redo").unwrap();
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
                            to_value(&PathArgs {
                                path: content_path.to_str().unwrap().to_string(),
                            })
                            .unwrap(),
                        )
                        .await
                        .as_string()
                        .unwrap();

                        if let Some(input_element) = text_input_ref.cast::<HtmlElement>() {
                            input_element.set_inner_text(content.as_str());

                            // gloo_console::log!("Setting Textarea content");
                            let _result =
                                input_element.dispatch_event(&InputEvent::new("input").unwrap());
                            // match result {
                            //     Ok(x) => gloo_console::log!(format!("{}", x)),
                            //     Err(x) => gloo_console::log!(x),
                            // }
                        }
                    });
                }
            } else {
                modal.set(html! {
                    <Modal
                        content={html! {
                <div class="flex flex-col h-32 w-full justify-between">
                    <div class="text-3xl text-center">
                        {"Please select or create a project"}
                    </div>
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
                            onclick={let open_modal2 = open_modal2.clone();
                            Callback::from(move |e: MouseEvent| {
                                open_modal2.emit(e);
                            })}
                        >
                            { "Create Project" }
                        </button>
                    </div>
                </div>
                                }}
                    />
                });
            }
        });
    }

    html! {
        <div class="h-screen w-screen flex flex-col">
            <div class="light lightdark medium dark verydark" />
            <div class="modal-wrapper">{ (*modal).clone() }</div>
            <style id="dynamic-style" />
            <Toolbar />
            <div class="h-8 flex justify-left items-center p-2 bg-crust">
                <Button
                    callback={open_modal}
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

        if invoke(
            "can_create_path",
            to_value(&PathArgs {
                path: path.to_string().clone(),
            })
            .unwrap(),
        )
        .await
        .as_string()
        .unwrap()
        .is_empty()
        {
            invoke(
                "create_directory",
                to_value(&PathArgs {
                    path: path.to_string().clone(),
                })
                .unwrap(),
            )
            .await;
        }

        let mut statistics_path = path.clone();
        statistics_path.push_str("/Statistics");

        if invoke(
            "can_create_path",
            to_value(&PathArgs {
                path: statistics_path.to_string().clone(),
            })
            .unwrap(),
        )
        .await
        .as_string()
        .unwrap()
        .is_empty()
        {
            invoke(
                "create_directory",
                to_value(&PathArgs {
                    path: statistics_path.to_string().clone(),
                })
                .unwrap(),
            )
            .await;
        }

        let settings_jsvalue = invoke(
            "get_settings",
            serde_wasm_bindgen::to_value(&PathArgs { path }).unwrap(),
        )
        .await;

        let settings_result = serde_wasm_bindgen::from_value::<Settings>(settings_jsvalue);

        let settings = settings_result.unwrap();

        let theme = settings.theme;

        println!("{theme:?}");

        let _ = Timeout::new(10, {
            move || {
                switch_theme(&theme);
            }
        })
        .forget();

        // switch_theme(theme.as_str());
    });
}

fn switch_theme(theme: &str) {
    let html_doc: HtmlDocument = document().dyn_into().unwrap();
    let body = html_doc.body().unwrap();
    let theme2 = theme.to_lowercase().replace(' ', "");
    body.set_class_name(format!("{theme2} bg-crust text-text").as_str());
}
