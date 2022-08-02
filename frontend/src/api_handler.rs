use crate::constants::API_ROUTE;
use common::UserInfo;
use reqwasm::http::Request;
use serde::de::DeserializeOwned;
use web_sys::console;

#[derive(Clone, PartialEq)]
pub struct ApiHandler {
    pub user_info: UserInfo,
    pub api_key: Option<String>,
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
            user_info: UserInfo { username, password },
            api_key: None,
        }
    }

    pub fn set_user_info(&mut self, user_info: UserInfo) {
        self.user_info = user_info;
    }

    pub fn get<T: 'static, U: 'static>(&self, route: String, action: U)
    where
        T: DeserializeOwned,
        U: Fn(T) -> (),
    {
        console::log_1(&"hejsan2".into());
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

    pub fn post<T: 'static, U: 'static>(&self, route: String, serialized_body: String, action: U)
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
}
