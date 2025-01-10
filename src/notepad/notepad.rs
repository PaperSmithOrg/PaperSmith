use pulldown_cmark::{html::push_html, Options, Parser};
use web_sys::HtmlElement;
use regex::Regex;
use yew::prelude::*;

#[path = "zoom_handlers.rs"]
mod zoom_edit_container_handlers;
use zoom_edit_container_handlers::ZoomControls;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub pages_ref: NodeRef,
    pub text_input_ref: NodeRef,
}

#[function_component(Notepads)]
pub fn notepads(
    Props {
        pages_ref,
        text_input_ref,
    }: &Props,
) -> Html {
    let zoom_compile_ref = use_node_ref();
    let zoom_edit_ref = use_node_ref();
    let font_size_edit = use_state(|| 16.0);
    let font_size_compile = use_state(|| 16.0);
    let render_ref = use_node_ref();
    let on_text_input = text_input_handler(text_input_ref.clone(), render_ref.clone());

    html!(
        <div class="flex flex-grow h-notepad bg-crust justify-evenly gap-5 px-3" ref={pages_ref.clone()}>
            <div
                class="bg-base max-h-full flex flex-1 flex-col overflow-hidden mx-2 rounded-md max-w-[45vw]"
            >
                <div
                    class="border-b-[2px] border-t-0 border-x-0 border-solid flex items-center px-2"
                >
                    <ZoomControls font_size={font_size_edit.clone()} container={zoom_edit_ref} />
                </div>
                <div
                    class="flex-grow p-4 overflow-x-hidden outline-none break-words"
                    id="notepad-textarea-edit"
                    ref={text_input_ref}
                    style={format!("font-size: {}px;", *font_size_edit)}
                    contenteditable="true"
                    oninput={on_text_input}
                />
            </div>
            <div
                class="bg-base max-h-full flex flex-1 flex-col overflow-hidden mx-2 rounded-md max-w-[45vw]"
            >
                <div
                    class="border-b-[2px] border-t-0 border-x-0 border-solid flex items-center px-2"
                >
                    <ZoomControls
                        font_size={font_size_compile.clone()}
                        container={zoom_compile_ref}
                    />
                </div>
                <div
                    class="flex-grow p-4 overflow-x-hidden break-words space-y-0"
                    id="notepad-textarea-compile"
                    style={format!("font-size: {}px; word-break: break-word;", *font_size_compile)}
                    ref={render_ref}
                />
            </div>
        </div>
    )
}
fn text_input_handler(text_input_ref: NodeRef, render_ref: NodeRef) -> Callback<InputEvent> {
    Callback::from(move |_| {
        if let Some(input) = text_input_ref.cast::<HtmlElement>() {
            let inner_text = input.inner_text();
            gloo_console::log!(&inner_text);
            let new_lines: Vec<String> = inner_text.lines().map(String::from).collect();
            //lines.set(new_lines);
            rendering_handler(&render_ref, &new_lines);
        }
    })
}

// ad br tag after end of each line (make it one string)
fn rendering_handler(render_ref: &NodeRef, new_lines: &[String]) {
    let mut last_was_empty = false;

    let mark_regex = Regex::new(r"::(.*?)::").unwrap();
    let underline_regex = Regex::new(r"__(.*?)__").unwrap();
    let image_regex = Regex::new(r"!\(\s*(.*?)\s*\)").unwrap();

    let html_strings: Vec<String> = new_lines
        .iter()
        .map(|line| {
            gloo_console::log!(line);
            let mut options = Options::empty();
            options.insert(Options::ENABLE_STRIKETHROUGH);
            options.insert(Options::ENABLE_TABLES);

            if line.trim().is_empty() {
                if last_was_empty {
                    "".to_string()
                } else {
                    last_was_empty = true;
                    "<br>".to_string()
                }
            } else {
                last_was_empty = false;

                let line_with_mark = mark_regex.replace_all(line, r#"<mark>$1</mark>"#);
                let line_with_underline = underline_regex.replace_all(&line_with_mark, r#"<u>$1</u>"#);
                let line_with_image = image_regex.replace_all(&line_with_underline, r#"<img src="$1"/>"#);

                let parser = Parser::new_ext(line_with_image.as_ref(), options);
                let mut html_output = String::new();
                push_html(&mut html_output, parser);
                html_output
            }
        })
        .collect();

    let html_string: String = html_strings.join("\n");

    if let Some(rendered) = render_ref.cast::<HtmlElement>() {
        rendered.set_inner_html(html_string.as_str());
    }
}
