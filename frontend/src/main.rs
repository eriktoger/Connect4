use gloo_events::EventListener;
use reqwasm::http::{Headers, Request};
use web_sys::{
    console, Event, EventSource, HtmlInputElement, InputEvent, MessageEvent, MouseEvent,
};
use yew::{function_component, html, use_effect_with_deps, use_state, Callback, Html};
use yew_router::prelude::{BrowserRouter, Routable, Switch};

use serde::{Deserialize, Serialize};
use wasm_bindgen::{JsCast, JsValue};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct GameInfo {
    room: String,
    message: String,
}

#[function_component(Home)]
fn home() -> Html {
    /*
    wasm_bindgen_futures::spawn_local(async move {
        let mut header = Headers::new();
        header.append("content-type", "text/plain");

        Request::post("http://localhost:8000/message2")
            .body("posting stuff")
            .send()
            .await
            .unwrap();

        let response = Request::get("http://localhost:8000/home")
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
        console::log_1(&response.into());

        let es = EventSource::new("http://localhost:8000/events").unwrap();

        let listener = EventListener::new(&es, "backend", move |event: &Event| {
            let e = event.dyn_ref::<MessageEvent>().unwrap();
            let text = e.data().as_string().unwrap();
            console::log_2(&"here: ".into(), &text.into())
        });
    });
    */

    let room = use_state(|| "2".to_string());
    let r2 = (*room).clone();
    let cb = Callback::from(move |_| {
        let k = r2.clone();
        let game_info = GameInfo {
            room: "1".to_string(),
            message: "2".to_string(),
        };
        let serialized = serde_json::to_string(&game_info).unwrap();
        wasm_bindgen_futures::spawn_local(async move {
            Request::post("http://localhost:8000/message2")
                .body(&serialized)
                .send()
                .await
                .unwrap();
        });
    });
    let room_copy = room.clone();
    let cb2 = Callback::from(move |event: Event| {
        room_copy.set(
            event
                .target()
                .unwrap()
                .unchecked_into::<HtmlInputElement>()
                .value(),
        );
    });
    let url = format!("{}{}", "http://localhost:8000/events/", *room);

    let es = use_state(|| EventSource::new(&url).unwrap());

    let listener = use_state(|| {
        EventListener::new(&es, "message", move |event: &Event| {
            let e = event.dyn_ref::<MessageEvent>().unwrap();
            let text = e.data().as_string().unwrap();
            console::log_2(&"here: ".into(), &text.into())
        })
    });

    let list_clone = listener.clone();
    let room_clone = room.clone();
    let room_dependicy = room.clone();
    use_effect_with_deps(
        move |_| {
            let url = format!("{}{}", "http://localhost:8000/events/", *room_clone);
            console::log_1(&url.clone().into());
            let new_es = EventSource::new(&url).unwrap();

            let new_listener = EventListener::new(&new_es, "message", move |event: &Event| {
                event.stop_propagation();
                let e = event.dyn_ref::<MessageEvent>().unwrap();
                let text = e.data().as_string().unwrap();
                console::log_2(&"here: ".into(), &text.into())
            });

            list_clone.set(new_listener);
            es.set(new_es);

            || ()
        },
        room_dependicy,
    );
    console::log_1(&(*listener.event_type()).into());
    let hej = &*room.clone();
    html! {
      <div>{"Home"}
      <button onclick={cb}> {"Send message"}</button>
      <span>{hej}</span>
      <input onchange={cb2}/>
      </div>
    }
}

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Home,
}

fn switch(routes: &Route) -> Html {
    match routes {
        Route::Home => html! {<Home/>},
    }
}

#[function_component(App)]
fn app() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={Switch::render(switch)} />
        </BrowserRouter>
    }
}

fn main() {
    yew::start_app::<App>();
}
