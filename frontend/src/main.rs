use web_sys::HtmlInputElement;
use yew::events::Event;
use yew::prelude::*;

#[function_component(App)]
fn app() -> Html {
    // let quote_of_the_day = "Much yew, much html, so much much";
    html! {
        <>
            // <h1 class="shadow-xl text-4xl font-bold underline">{ "Hello Everybody" }</h1>
            // <h2 class="text-orange-600 text-3xl">{ quote_of_the_day }</h2>
            <Configuration />
        </>
    }
}

#[function_component(Navbar)]
fn navbar() -> Html {
    html! {
        <nav class="h-12 flex flex-row items-center justify-between shadow-xl mb-4">
            <h1 class="p-4">{"AI Frame"}</h1>
            <ul class="flex flex-row items-center p-4 space-x-4">
                <li>{"Button 1"}</li>
                <li>{"Button 2"}</li>
            </ul>
        </nav>
    }
}

#[derive(Properties, PartialEq)]
struct OptionCardProps {
    title: String,
    details: String,
    #[prop_or_default]
    children: Children,
}

#[function_component(OptionCard)]
fn option_card(
    OptionCardProps {
        title,
        details,
        children,
    }: &OptionCardProps,
) -> Html {
    html! {
        <div class="flex flex-row items-center p-4 space-x-4 justify-between">
            <div class="shrink space-y-2">
                <h5 class="text-slate-800 text-2xl font-bold">{title}</h5>
                <p class="text-slate-500">{details}</p>
            </div>
            <div class="min-w-fit">
                {for children.iter()}
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct ToggleProps {
    on_toggle: Callback<bool>,
}

#[function_component(Toggle)]
fn toggle(ToggleProps{on_toggle}: &ToggleProps) -> Html {
    let checkbox_ref = use_node_ref();

    let handle_change = {
        let checkbox_ref = checkbox_ref.clone();
        let on_toggle = on_toggle.clone();
        Callback::from(move |_| {
            if let Some(checkbox) = checkbox_ref.cast::<HtmlInputElement>() {
                on_toggle.emit(checkbox.checked());
            }
        })
    };

    html! {
        <label for="default-toggle" class="inline-flex relative items-center cursor-pointer">
            <input type="checkbox" value="" id="default-toggle" class="sr-only peer" onchange={handle_change} ref={checkbox_ref} />
            <div class="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-blue-300 dark:peer-focus:ring-blue-800 rounded-full peer dark:bg-gray-700 peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all dark:border-gray-600 peer-checked:bg-blue-600"></div>
            <span class="ml-3 text-sm font-medium text-gray-900 dark:text-gray-300">{"Toggle me"}</span>
        </label>
    }
}

#[function_component(Configuration)]
fn configuration() -> Html {
    let handle_toggle = Callback::from(|state: bool| {
        gloo_console::log!("Toggled: ", state);
    });

    html! {
        <>
            <Navbar />
            <OptionCard
                title="Don't do something"
                details="Some really nice details about doing something or not!"
            >
                <Toggle on_toggle={handle_toggle.clone()}/>
            </OptionCard>
            <OptionCard
                title="Do something"
                details="I want to do something about this now!"
            >
                <Toggle on_toggle={handle_toggle.clone()}/>
            </OptionCard>
        </>
    }
}

fn main() {
    yew::start_app::<App>();
}
