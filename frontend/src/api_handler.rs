use crate::constants::API_ROUTE;
use common::UserInfo;
use gloo_events::EventListener;
use reqwasm::http::Request;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use wasm_bindgen::JsCast;
use web_sys::{Event, EventSource, MessageEvent};

#[derive(Serialize, Deserialize)]
struct UserAuth {
    api_key: String,
    username: String,
}
#[derive(Clone, PartialEq)]
pub struct ApiHandler {
    pub user_info: UserInfo,
}

impl ApiHandler {
    pub fn new() -> ApiHandler {
        let local_storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();

        let username = local_storage
            .get_item("username")
            .unwrap()
            .unwrap_or_default();

        let password = local_storage
            .get_item("password")
            .unwrap()
            .unwrap_or_default();

        ApiHandler {
            user_info: UserInfo {
                username,
                password,
                api_key: None,
            },
        }
    }

    pub fn get<T: 'static, U: 'static>(route: String, action: U)
    where
        T: DeserializeOwned,
        U: Fn(T) -> (),
    {
        wasm_bindgen_futures::spawn_local(async move {
            let url = format!("{}{}", API_ROUTE, route);

            let response = Request::get(&url)
                .send()
                .await
                .unwrap()
                .json()
                .await
                .unwrap();
            action(response);
        });
    }

    pub fn auth_get<T: 'static, U: 'static>(&self, route: String, action: U)
    where
        T: DeserializeOwned,
        U: Fn(T) -> (),
    {
        let user_auth = UserAuth {
            username: self.user_info.username.clone(),
            api_key: self.user_info.api_key.clone().unwrap_or_default(),
        };
        let serialized = serde_json::to_string(&user_auth).unwrap();
        wasm_bindgen_futures::spawn_local(async move {
            let url = format!("{}{}", API_ROUTE, route);
            let response = Request::get(&url)
                .header("x-api-key", &serialized)
                .send()
                .await
                .unwrap()
                .json()
                .await
                .unwrap();
            action(response);
        });
    }

    pub fn post<T: 'static, U: 'static>(route: String, serialized_body: String, action: U)
    where
        T: DeserializeOwned + Default,
        U: Fn(T) -> (),
    {
        wasm_bindgen_futures::spawn_local(async move {
            let url = format!("{}{}", API_ROUTE, route);
            let response = Request::post(&url)
                .body(serialized_body)
                .send()
                .await
                .unwrap()
                .json()
                .await
                .unwrap();
            action(response)
        });
    }

    pub fn auth_post<T: 'static, U: 'static>(
        &self,
        route: String,
        serialized_body: String,
        action: U,
    ) where
        T: DeserializeOwned,
        U: Fn(T) -> (),
    {
        let api_key = self.user_info.api_key.clone().unwrap_or_default();
        let user_auth = UserAuth {
            username: self.user_info.username.clone(),
            api_key,
        };
        let serialized = serde_json::to_string(&user_auth).unwrap();
        wasm_bindgen_futures::spawn_local(async move {
            let url = format!("{}{}", API_ROUTE, route);
            let response = Request::post(&url)
                .body(serialized_body)
                .header("x-api-key", &serialized)
                .send()
                .await
                .unwrap()
                .json()
                .await
                .unwrap();
            action(response)
        });
    }

    pub fn get_event_listener<T: 'static, U: 'static>(route: String, action: U) -> EventListener
    where
        T: DeserializeOwned,
        U: Fn(T) -> (),
    {
        let url = format!("{}{}", API_ROUTE, route);
        let es = EventSource::new(&url).unwrap();
        EventListener::new(&es, "message", move |event: &Event| {
            let e = event.dyn_ref::<MessageEvent>().unwrap();
            let text = e.data().as_string().unwrap();
            let deserialized: T = serde_json::from_str(&text).unwrap();
            action(deserialized);
        })
    }
}
