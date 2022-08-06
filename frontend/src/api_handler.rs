use crate::constants::API_ROUTE;
use common::UserInfo;
use gloo_events::EventListener;
use reqwasm::http::Request;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::error::Error;
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
        let (username, password) = match ApiHandler::check_local_storage() {
            Ok((usernamne, password)) => (usernamne, password),
            Err(_) => ("".to_string(), "".to_string()),
        };

        ApiHandler {
            user_info: UserInfo {
                username,
                password,
                api_key: None,
            },
        }
    }

    pub fn get<T: 'static, U: 'static, V: 'static>(route: String, on_success: U, on_failure: V)
    where
        T: DeserializeOwned,
        U: Fn(T) -> (),
        V: Fn() -> (),
    {
        wasm_bindgen_futures::spawn_local(async move {
            let url = format!("{}{}", API_ROUTE, route);
            match Request::get(&url).send().await {
                Ok(response) => match response.json().await {
                    Ok(val) => on_success(val),
                    Err(_) => on_failure(),
                },
                Err(_) => on_failure(),
            }
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

    fn check_local_storage() -> Result<(String, String), Box<dyn Error>> {
        let local_storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();

        let username_result = local_storage.get_item("username");
        let username = match username_result {
            Ok(val) => match val {
                Some(val) => val,
                None => {
                    return Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "Username not found",
                    )))
                }
            },
            Err(_) => {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Username not found",
                )))
            }
        };

        let password_result = local_storage.get_item("password");
        let password = match password_result {
            Ok(val) => match val {
                Some(val) => val,
                None => {
                    return Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "Password not found",
                    )))
                }
            },
            Err(_) => {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Password not found",
                )))
            }
        };
        Ok((username, password))
    }
}
