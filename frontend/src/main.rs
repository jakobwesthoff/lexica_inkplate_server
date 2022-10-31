use yew::prelude::*;

#[function_component(App)]
fn app() -> Html {
    html! {
        <h1 class={"text-4xl font-bold underline"}>{ "Hello Youtube!" }</h1>
    }
}

fn main() {
    yew::start_app::<App>();
}
