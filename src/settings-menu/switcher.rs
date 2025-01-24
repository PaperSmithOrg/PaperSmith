use web_sys::HtmlSelectElement;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub theme: String,
}

#[function_component(ThemeSwitcher)]
pub fn switcher(Props { theme }: &Props) -> Html {
    let theme = theme.clone();

    let switch_ref = use_node_ref();

    let themes = [
        "Light".to_string(),
        "Light Dark".to_string(),
        "Medium".to_string(),
        "Dark".to_string(),
        "Very Dark".to_string(),
    ];

    let onchange = {
        let select_ref = switch_ref.clone();
        let theme = theme.clone();

        Callback::from(move |_| {
            let select = select_ref.cast::<HtmlSelectElement>();
            let mut theme = theme.clone();

            if let Some(select) = select {
                theme = select.value();
            }
        })
    };

    let themes_vec = themes_to_html(Vec::from(themes.clone()));

    html! {
        <div>
            <div>
                <select ref={switch_ref}
                        onchange={onchange}
                        class="bg-base rounded-lg text-text focus:ring-secondary border-1 border-primary"
                >
                    { themes_vec }
                </select>
            </div>
        </div>
    }
}

fn themes_to_html(themes: Vec<String>) -> Html {
    themes
        .iter()
        .map(|theme| {
            // gloo_console::log!(theme.clone());
            html! {
                <option value={theme.clone()}
                >
                { theme }
                </option>
            }
        })
        .collect()
}

async fn write_changes(theme: String) {
    gloo_console::log!(format!("theme to write: {}", theme));

    let content = json!({
        "theme": theme,
    })
    .to_string();

    let name = String::from("settings");

    let path_jsvalue = invoke("get_data_dir", JsValue::null()).await;

    let mut path = path_jsvalue.as_string().expect("Cast failed").clone();

    path.push_str("/PaperSmith");

    gloo_console::log!(path.clone());

    let settings = FileWriteData {
        path,
        name,
        content,
    };

    invoke(
        "write_to_json",
        serde_wasm_bindgen::to_value(&settings).unwrap(),
    )
    .await;
}

fn switch_theme(theme: String) {
    gloo_console::log!(format!("new theme: {}", theme));

    let html_doc: HtmlDocument = document().dyn_into().unwrap();
    let body = html_doc.body().unwrap();
    let theme2 = theme.to_lowercase().replace(' ', "");
    body.set_class_name(format!("{theme2} bg-crust text-text").as_str());
}
