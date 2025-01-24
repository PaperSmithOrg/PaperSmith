use gloo::utils::document;
use gloo_timers::callback::Timeout;
use pulldown_cmark::{html, Options, Parser};
use serde::{Serialize, Deserialize};
use serde_wasm_bindgen::to_value;
use sidebar::buttons::Button;
use statistic::StatisticWindow;
use statistic::_CharCountProps::closing_callback;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlDocument;
use web_sys::HtmlElement;
use web_sys::Node;
use yew::events::InputEvent;
use yew::events::MouseEvent;
use yew::prelude::*;
use yew_icons::IconId;

use shared::Settings;


#[path = "notepad/notepad.rs"]
mod notepad;
use notepad::Notepads;

#[path = "toolbar/toolbar.rs"]
mod toolbar;
use toolbar::Toolbar;
use yew_hooks::use_interval;
use std::path::PathBuf;

// #[path = "theme-switcher/switcher.rs"]
// mod switcher;
// use switcher::ThemeSwitcher;

//#[path = "text_alignment_handlers.rs"]
//mod text_alignment_handlers;
//use text_alignment_handlers::TextAlignmentControls;

#[path = "menubar/text/text_styling_handlers.rs"]
mod text_styling_handlers;
use text_styling_handlers::TextStylingControls;

#[path = "statistics/statistic.rs"]
mod statistic;
use statistic::Statistics;

//#[path = "text_alignment_handlers.rs"]
//mod text_alignment_handlers;
//use text_alignment_handlers::TextAlignmentControls;

#[path = "sidebar/sidebar.rs"]
mod sidebar;
use shared::Project;
use sidebar::SideBar;

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
    pub content: String
}

// #[derive(Properties, PartialEq)]
// pub struct StatisticProps {
//     pub statistics: StatisticProp,
// }

#[function_component(App)]
pub fn app() -> Html {
    let project: UseStateHandle<Option<Project>> = use_state(|| None);
    let sidebar = use_state(|| {
        html! {
            <div class="text-lg">{ "No Project Loaded" }</div>
        }
    });
    let modal = use_state(|| html!());

    let project_path = project.as_ref().map(|proj| proj.path.clone());
    let text_input_ref = use_node_ref();
    let pages_ref = use_node_ref();
    
    {
        use_effect_with((), move |_| {
            apply_settings();
        });
    }

    let save_fn = {
        let text_input_ref = text_input_ref.clone();
        let project_path = project_path.clone();
        let modal = modal.clone();

        Callback::from(move |_| {
            let text_input_ref = text_input_ref.clone();
            let project_path = project_path.clone();
            let modal = modal.clone();

            spawn_local(async move {
                if let Some(input_element) = text_input_ref.cast::<HtmlElement>() {
                    let text = input_element.inner_text();

                    if let Some(mut path) = project_path {
                        path.push("Chapters");
                        path.push("Beginning");
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

                        modal.set(html! {
                            /*
                            <Modal
                                content={html! {
                                    <div>{ "Successfully saved" }</div>
                                }}
                                button_configs={
                                    vec![
                                        ModalButtonProps {
                                            text: "Close".to_string(),
                                            text_color: "white".to_string(),
                                            bg_color: "green".to_string(),
                                            callback: {
                                                let modal = modal.clone();
                                                Callback::from(move |_| modal.set(html!()))
                                            }
                                        }
                                    ]
                                }
                            />
                            */
                        });
                    }
                }
            });
        })
    };

    let save = {
        let save_fn = save_fn.clone();
        Callback::from(move |_| save_fn.emit(()))
    };

    {
        let save = save_fn.clone();
        use_interval(move || {
            save.emit(());
        }, 300_000); // 300,000 ms = 5 minutes
    }

    let open_modal = {
        let modal = modal.clone();
        let project = project.clone();
        Callback::from(move |_| {
            modal.set(html! {
                <Modal
                    content={html! {
                        <ProjectWizard
                            closing_callback={
                                let modal = modal.clone();
                                Callback::from(move |_| modal.set(html!()))
                            }
                            project_ref={project.clone()}
                        />
                    }}
                />
            });
        })
    };

    let open_statistics = {
        let modal = modal.clone();
        let pages_ref = pages_ref.clone();
        Callback::from(move |_| {
            modal.set(html! {
                <VerticalModal
                    content={html! {
                        <StatisticWindow
                            closing_callback={
                                let modal = modal.clone();
                                Callback::from(move |_| modal.set(html!()))
                            }
                            pages_ref={pages_ref.clone()}
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


    {
        let sidebar = sidebar.clone();
        let project = project.clone();
        let text_input_ref = text_input_ref.clone();
        use_effect_with(project.clone(), move |_| {
            if project.is_none() {
                sidebar.set(html! {
                    <div class="cursor-default select-none text-lg">{ "No Project Loaded" }</div>
                });
            } else {
                sidebar.set(
                    html! { <SideBar project={project.clone()} input_ref={text_input_ref.clone()} /> },
                );
            }
        });
    }

    let on_load = {
        let project = project.clone();
        Callback::from(move |_| {
            let project = project.clone();
            spawn_local(async move {
                let project_jsvalue = invoke("get_project", JsValue::null()).await;
                let project_or_none: Option<Project> =
                    serde_wasm_bindgen::from_value(project_jsvalue).unwrap();
                if project_or_none.is_some() {
                    project.set(project_or_none);
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

    html! {
        <div class="h-screen w-screen flex flex-col">
            <div class="light lightdark medium dark verydark" />
            <div class="modal-wrapper">{ (*modal).clone() }</div>
            <style id="dynamic-style" />
            <Toolbar />
            <div class="h-12 flex justify-left items-center p-2 bg-crust">
                <Button callback={open_modal} icon={IconId::LucideFilePlus} title="Create Project" size=1.5 />
                <Button callback={on_load} icon={IconId::LucideFolderOpen} title="Load Project" size=1.5 />
                <Button callback={save} icon={IconId::LucideSave} title="Save" size=1.5 />
                <Button callback={open_settings} icon={IconId::LucideSettings} title={"Open Settings"} size=1.5 />
                <div class="w-[1px] h-[20px] bg-subtext my-0 mx-1 " />
                <Button callback={on_undo} icon={IconId::LucideUndo} title="Undo" size=1.5 />
                <Button callback={on_redo} icon={IconId::LucideRedo} title="Redo" size=1.5 />
                <div class="w-[1px] h-[20px] bg-subtext my-0 mx-1 " />
                <TextStylingControls />
            </div>

            <div id="main_content" class="flex flex-grow m-3">
                <div class="flex flex-col min-w-[18rem] overflow-y-auto bg-crust">
                    <div class="flex-grow">{ (*sidebar).clone() }</div>
                    // <div class="bottom-5 left-2 right-2">
                    //     <ThemeSwitcher />
                    // </div>
                </div>
                <Notepads pages_ref={pages_ref.clone()} text_input_ref={text_input_ref} />
            </div>
            <div
                class="h-3 justify-between items-center flex p-2 bg-crust border-solid border-t-[2px] border-x-0 border-b-0 border-text"
            >
                <div class="bottombar-left">
                    <Statistics
                        closing_callback={
                            let modal = modal.clone();
                            Callback::from(move |_| modal.set(html!()))
                        }
                        pages_ref={pages_ref.clone()}
                    />
                </div>
                <div class="bottombar-right">
                <Button callback={open_statistics} icon={IconId::LucideBarChart3} title="Statistics" size=1.5/>
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

        let settings_jsvalue = invoke(
            "get_settings",
            serde_wasm_bindgen::to_value(&PathArgs { path: path }).unwrap(),
        )
        .await;

        let settings_result = serde_wasm_bindgen::from_value::<Settings>(settings_jsvalue);

        let settings = settings_result.unwrap();

        let theme = settings.theme;

        let _ = Timeout::new(10, {
            let theme = theme.clone();

            move || {
                switch_theme(theme);
            }
        })
        .forget();
    });
}

fn switch_theme(theme: String) {
    let html_doc: HtmlDocument = document().dyn_into().unwrap();
    let body = html_doc.body().unwrap();
    let theme2 = theme.to_lowercase().replace(' ', "");
    body.set_class_name(format!("{theme2} bg-crust text-text").as_str());
}

/*let save = Callback::from(move |_: MouseEvent| {
    let args = to_value(&()).unwrap();
    let ahhh = invoke("show_save_dialog", args).await;
});*/

/*This one worked----------------------------------------------------------
let save = {
    Callback::from(move |_| {
        spawn_local(async move {
            let args = to_value(&()).unwrap();
            let ahhh = invoke("show_save_dialog", args).await;
        });
    })
};*/

/*let save = {
    Callback::from(move |_| {
        spawn_local(async move {
            let args = to_value(&()).unwrap();
            invoke("saveTest", args).await.as_string();
        });
    })
};*/
