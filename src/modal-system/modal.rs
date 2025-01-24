use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub content: Html,
}

#[function_component(Modal)]
pub fn modal(Props { content }: &Props) -> Html {
    html!(
        <>
            <div
                class="absolute top-0 left-0 z-50 bg-black/60 h-full w-full flex items-center justify-center text-text"
            >
                <div class="bg-base rounded-lg max-w-[80%] min-w-[80%] p-8">
                    { content.clone() }
                </div>
            </div>
        </>
    )
}

#[function_component(VerticalModal)]
pub fn vertical_modal(Props { content }: &Props) -> Html {
    html!(
        <>
            <div
                class="absolute top-0 left-0 z-50 bg-black/60 h-full w-full flex items-center justify-center text-text"
            >
                <div
                    class="bg-base rounded-lg max-w-[30%] min-w-[30%] max-h-[50%] min-h-[50%] p-8 flex flex-col justify-between"
                >
                    { content.clone() }
                </div>
            </div>
        </>
    )
}
