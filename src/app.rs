use std::rc::Rc;

use gloo::utils::document;
use serde::{Deserialize, Serialize};
use shared::Project;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlDocument;
use web_sys::HtmlElement;
use yew::prelude::*;
use yew_hooks::use_interval;
use yew_icons::IconId;
use yewdux::prelude::*;

#[path = "notepad/notepad.rs"]
mod notepad;
use notepad::Notepads;

#[path = "toolbar/toolbar.rs"]
mod toolbar;
use toolbar::Toolbar;

#[path = "theme-switcher/switcher.rs"]
mod switcher;
use switcher::ThemeSwitcher;

//#[path = "text_alignment_handlers.rs"]
//mod text_alignment_handlers;
//use text_alignment_handlers::TextAlignmentControls;

#[path = "menubar/text/text_styling_handlers.rs"]
mod text_styling_handlers;
use text_styling_handlers::TextStylingControls;

#[path = "statistics/statistic.rs"]
mod statistic;
use statistic::StatisticWindow;
use statistic::Statistics;

//#[path = "text_alignment_handlers.rs"]
//mod text_alignment_handlers;
//use text_alignment_handlers::TextAlignmentControls;

#[path = "sidebar/sidebar.rs"]
mod sidebar;
use sidebar::buttons::Button;
use sidebar::SideBarWrapper;

#[path = "project-wizard/wizard.rs"]
mod wizard;
use wizard::ProjectWizard;

#[path = "modal-system/modal.rs"]
mod modal;
use modal::Modal;
use modal::VerticalModal;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "tauri"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[derive(Properties, PartialEq)]
pub struct WordCountProps {
    pub pages_ref: NodeRef,
}

#[derive(Serialize, Deserialize)]
pub struct FileWriteData {
    pub path: String,
    pub content: String,
}

#[derive(Default, Clone, PartialEq, Eq, Store, Debug)]
pub struct State {
    project: Option<Project>,
}

// #[derive(Properties, PartialEq)]
// pub struct StatisticProps {
//     pub statistics: StatisticProp,
// }

#[function_component(App)]
pub fn app() -> Html {
    let (state, dispatch) = use_store::<State>();
    let modal = use_state(|| html!());

    let project_path = state.project.as_ref().map(|proj| proj.path.clone());
    let text_input_ref = use_node_ref();
    let pages_ref = use_node_ref();

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
                        // TODO: change to use active chapter name
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
        use_interval(
            move || {
                save.emit(());
            },
            300_000,
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

    let on_load = {
        Callback::from(move |_| {
            let dispatch = dispatch.clone();
            spawn_local(async move {
                let project_jsvalue = invoke("get_project", JsValue::null()).await;
                let project_or_none: Option<Project> =
                    serde_wasm_bindgen::from_value(project_jsvalue).unwrap();
                if project_or_none.is_some() {
                    dispatch.set(State {
                        project: project_or_none,
                    });
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
                <div class="w-[1px] h-[20px] bg-subtext my-0 mx-1 " />
                <Button callback={on_undo} icon={IconId::LucideUndo} title="Undo" size=1.5 />
                <Button callback={on_redo} icon={IconId::LucideRedo} title="Redo" size=1.5 />
                <div class="w-[1px] h-[20px] bg-subtext my-0 mx-1 " />
                <TextStylingControls />
            </div>
            <div id="main_content" class="flex flex-grow m-3">
                <div class="flex flex-col min-w-[18rem] overflow-y-auto bg-crust">
                    <div class="flex-grow"> {html!{<SideBarWrapper input_ref={text_input_ref.clone()} />}}</div>
                    <div class="bottom-5 left-2 right-2">
                        <ThemeSwitcher />
                    </div>
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
