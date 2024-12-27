use web_sys::HtmlSelectElement;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub theme: UseStateHandle<String>,
}

#[function_component(ThemeSwitcher)]
pub fn switcher(    Props {
    theme,
}: &Props,) -> Html {
    let theme = theme;

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

            if let Some(select) = select {
                theme.set(select.value());
                // switch_theme(theme.to_string().to_lowercase().replace(' ', ""));
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

// fn switch_theme(theme: String) {
//     let html_doc: HtmlDocument = document().dyn_into().unwrap();
//     let body = html_doc.body().unwrap();
//     gloo_console::log!(theme.clone());
//     body.set_class_name(format!("{theme} bg-crust text-text").as_str());
// }

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