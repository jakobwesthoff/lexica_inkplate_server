use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Debug, Deserialize, Serialize, Clone)]
struct PersistedConfig {
    update_at_night: bool,
    update_interval: usize,
}

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
        <nav class="h-14 shadow-md mb-7 border-b-2 border-purple-700 shadow-purple-700/25">
            <div class="min-h-full mx-auto max-w-screen-md flex flex-row items-center justify-between">
                <a href="/" class="flex items-center ml-4 md:ml-0">
                    <img src="images/logo-128.png" class="h-8 mr-4" alt="AI Frame Logo" />
                    <h1 class="text-2xl font-semibold whitespace-nowrap">{"AI Frame"}</h1>
                </a>
                <ul class="flex flex-row items-center space-x-4 mr-4 md:mr-0">
                    <li>{"Button 1"}</li>
                    <li>{"Button 2"}</li>
                </ul>
            </div>
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
        <li class="flex flex-row items-center p-4 space-x-4 justify-between">
            <div class="shrink space-y-2">
                <h5 class="text-slate-800 text-2xl font-bold">{title}</h5>
                <p class="text-slate-500">{details}</p>
            </div>
            <div class="min-w-fit">
                {for children.iter()}
            </div>
        </li>
    }
}

#[derive(PartialEq)]
enum ToggleSize {
    Small,
    Medium,
    Large,
}

#[derive(Properties, PartialEq)]
struct ToggleProps {
    on_toggle: Callback<bool>,
    size: Option<ToggleSize>,
    text: Option<String>,
}

#[function_component(Toggle)]
fn toggle(
    ToggleProps {
        on_toggle,
        size,
        text,
    }: &ToggleProps,
) -> Html {
    let size = size.as_ref().unwrap_or(&ToggleSize::Medium);

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

    let size_classes = match size {
        ToggleSize::Small => vec![
            "w-9",
            "h-5",
            "after:top-[2px]",
            "after:left-[2px]",
            "after:h-4",
            "after:w-4",
        ],
        ToggleSize::Medium => vec![
            "w-11",
            "h-6",
            "after:top-[2px]",
            "after:left-[2px]",
            "after:h-5",
            "after:w-5",
        ],
        ToggleSize::Large => vec![
            "w-14",
            "h-7",
            "after:top-0.5",
            "after:left-[4px]",
            "after:h-6",
            "after:w-6",
        ],
    };

    let uuid_ref = use_ref(|| Uuid::new_v4());
    let uuid_str = format!("toggle-{}", uuid_ref);

    html! {
        <label for={uuid_str.clone()} class="inline-flex relative items-center cursor-pointer">
            <input type="checkbox" value="" id={uuid_str.clone()} class="sr-only peer" onchange={handle_change} ref={checkbox_ref} />
            <div class={classes!(size_classes, "bg-gray-200", "peer-focus:outline-none", "peer-focus:ring-4", "peer-focus:ring-purple-300", "rounded-full", "peer", "peer-checked:after:translate-x-full", "peer-checked:after:border-white", "after:content-['']", "after:absolute", "after:bg-white", "after:border-gray-300", "after:border", "after:rounded-full", "after:transition-all", "peer-checked:bg-purple-600")}></div>
            if let Some(text) = text {
                <span class="ml-3 text-sm font-medium text-gray-900 dark:text-gray-300">{text}</span>
            }
        </label>
    }
}

#[derive(Properties, PartialEq)]
struct OptionListProps {
    children: Children,
}

#[function_component(OptionList)]
fn option_list(OptionListProps { children }: &OptionListProps) -> Html {
    html! {
        <ul class="divide-y divide-slate-300 max-w-screen-md md:shadow-md md:border md:rounded-lg md:border-slate-200 mx-auto">
            {for children.iter()}
        </ul>
    }
}

#[function_component(Configuration)]
fn configuration() -> Html {
    let handle_toggle = Callback::from(|state: bool| {
        gloo_console::log!("Toggled: ", state);
    });

    let handle_button_click = Callback::from(|_| {
        gloo_console::log!("Button clicked.");
        wasm_bindgen_futures::spawn_local(async move {
            let result = Request::get("/api/config").send().await.unwrap();
            let config = result.json::<PersistedConfig>().await.unwrap();
            gloo_console::log!(format!("{:?}", config));
        });
    });

    html! {
        <>
            <Navbar />
            <OptionList>
                <OptionCard
                    title="Don't do something"
                    details="Some really nice details about doing something or not!"
                >
                    <Toggle on_toggle={handle_toggle.clone()} size={ToggleSize::Large}/>
                </OptionCard>
                <OptionCard
                    title="Do something"
                    details="I want to do something about this now!"
                >
                    <Toggle on_toggle={handle_toggle.clone()} size={ToggleSize::Large} />
                </OptionCard>
            </OptionList>
            <button onclick={handle_button_click}>{"Press Me!"}</button>
        </>
    }
}

fn main() {
    yew::start_app::<App>();
}
