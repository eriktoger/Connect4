use gloo_events::EventListener;
use reqwasm::http::Request;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;
use web_sys::{console, Event, EventSource, HtmlInputElement, MessageEvent};
use yew::{function_component, html, use_effect_with_deps, use_state, Callback};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct GameInfo {
    room: String,
    message: String,
}

#[function_component(Home)]
pub fn home() -> Html {
    let room = use_state(|| "2".to_string());
    let cb = Callback::from(move |_| {
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
