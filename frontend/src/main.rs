use yew::prelude::*;

#[function_component(App)]
fn app() -> Html {
    let quote_of_the_day = "Much yew, much html, so much much";
    html! {
        <>
            <h1 class="shadow-xl text-4xl font-bold underline">{ "Hello YouTube" }</h1>
            <h2 class="text-orange-600 text-3xl">{ quote_of_the_day }</h2>
        </>
    }
}

fn main() {
    yew::start_app::<App>();
}
