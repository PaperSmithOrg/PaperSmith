use web_sys::HtmlElement;
use yew::prelude::*;
use yew_icons::IconId;

use crate::app::sidebar::buttons::Button;

pub fn zoom_increase_handler(
    font_size: UseStateHandle<f64>,
    container_ref: NodeRef,
) -> Callback<MouseEvent> {
    let max_font_size = 72.0;

    Callback::from(move |_| {
        // Get the current font size and increase it by 1
        let current_font_size = *font_size;
        let new_font_size = (current_font_size + 1.0).min(max_font_size);
        font_size.set(new_font_size);

        // Apply the new font size to the container using inline styles
        if let Some(container) = container_ref.cast::<HtmlElement>() {
            container.set_inner_html(&format!("font-size: {new_font_size}px;"));
        }
    })
}

pub fn zoom_decrease_handler(
    font_size: UseStateHandle<f64>,
    container_ref: NodeRef,
) -> Callback<MouseEvent> {
    let min_font_size = 5.0;

    Callback::from(move |_| {
        // Get the current font size and decrease it by 1
        let current_font_size = *font_size;
        let new_font_size = (current_font_size - 1.0).max(min_font_size);
        font_size.set(new_font_size);

        // Apply the new font size to the container using inline styles
        if let Some(container) = container_ref.cast::<HtmlElement>() {
            container.set_inner_html(&format!("font-size: {min_font_size}px;"));
        }
    })
}

#[derive(Properties, PartialEq)]
pub struct ZoomProps {
    pub font_size: UseStateHandle<f64>,
    pub container: NodeRef,
}

// Component to render the zoome controls
#[function_component(ZoomControls)]
pub fn zoom_controls(
    ZoomProps {
        font_size,
        container,
    }: &ZoomProps,
) -> Html {
    // Handlers for increasing and decreasing the toom
    let on_zoom_increase = zoom_increase_handler(font_size.clone(), container.clone());
    let on_zoom_decrease = zoom_decrease_handler(font_size.clone(), container.clone());

    // Render the controls with two buttons
    html! {
        <div class="subbar-icon flex items-center m-1 select-none">
            <Button
                callback={on_zoom_decrease}
                icon={IconId::LucideZoomOut}
                title="Zoom Out"
                size=2.5
            />
            <Button
                callback={on_zoom_increase}
                icon={IconId::LucideZoomIn}
                title="Zoom In"
                size=2.5
            />
        </div>
    }
}
