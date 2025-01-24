use web_sys::{window, Range};
use yew::prelude::*;
use yew_hooks::use_interval;
use yew_icons::IconId;

use crate::app::sidebar::buttons::Button;

fn apply_style(range: &Range, style: &String) {
    let document = window().unwrap().document().unwrap();

    let selected_text = range.to_string();

    let new_text = format!("{style}{selected_text}{style}");

    let text_node = document.create_text_node(&new_text);
    range.delete_contents().unwrap();
    range.insert_node(&text_node).unwrap();

    // Clear the current selection
    let window = window().unwrap();
    let selection = window.get_selection().unwrap().unwrap();
    selection.remove_all_ranges().unwrap();
    selection.add_range(range).unwrap();
    range.collapse();
}

#[derive(Properties, PartialEq)]
pub struct TextStylingProps {
    pub text_styling: UseStateHandle<String>,
}

#[derive(Properties, PartialEq)]
pub struct StyleButtonProps {
    pub icon: IconId,
    pub title: String,
    pub range: UseStateHandle<Option<Range>>,
    pub style: String,
}

#[function_component(StyleButton)]
pub fn style_button(style_props: &StyleButtonProps) -> Html {
    let range_state = style_props.range.clone();
    let style = style_props.style.clone();

    let onclick = Callback::from(move |_| {
        if let Some(range) = range_state.as_ref() {
            apply_style(range, &style);

            let document = gloo::utils::document();
            if let Some(notepad_element) = document.get_element_by_id("notepad-textarea-edit") {
                let event = web_sys::InputEvent::new("input").unwrap();
                notepad_element.dispatch_event(&event).unwrap();
            }
        }
    });

    html! {
        <Button
            callback={onclick}
            icon={style_props.icon}
            title={style_props.title.clone()}
            size=1.5
        />
    }
}

#[function_component(TextStylingControls)]
pub fn text_styling_controls() -> Html {
    let range_state = use_state(|| None);
    let inner_range_state = range_state.clone();
    use_interval(
        move || {
            let window = window().unwrap();
            let document = window.document().unwrap();

            if let Some(selection) = document.get_selection().unwrap() {
                if let Ok(range) = selection.get_range_at(0) {
                    let common_ancestor = range.common_ancestor_container().unwrap();
                    let notepad = document.get_element_by_id("notepad-textarea-edit").unwrap();

                    let mut is_within = false;
                    let mut node = common_ancestor;
                    while let Some(parent) = node.parent_node() {
                        if parent.is_same_node(Some(&notepad)) {
                            is_within = true;
                            break;
                        }
                        node = parent;
                    }

                    if is_within {
                        if let Ok(range) = selection.get_range_at(0) {
                            inner_range_state.set(Some(range));
                        }
                    }
                }
            }
        },
        10,
    );

    let stylebutton_props = vec![
        StyleButtonProps {
            icon: IconId::LucideBold,
            title: "Bold".to_string(),
            range: range_state.clone(),
            style: "**".to_string(),
        },
        StyleButtonProps {
            icon: IconId::LucideItalic,
            title: "Italic".to_string(),
            range: range_state.clone(),
            style: "_".to_string(),
        },
        StyleButtonProps {
            icon: IconId::LucideUnderline,
            title: "Underline".to_string(),
            range: range_state.clone(),
            style: "__".to_string(),
        },
        StyleButtonProps {
            icon: IconId::LucideHighlighter,
            title: "Highlighter".to_string(),
            range: range_state,
            style: "::".to_string(),
        },
    ];

    html! {
        <div class="flex">
            { stylebutton_props
            .iter()
            .map(|props| {
                html! { <>
                    <StyleButton icon={props.icon} title={props.title.clone()} range={props.range.clone()} style={props.style.clone()}/>
                    </>
                }
            })
            .collect::<Html>() }
        </div>
    }
}
