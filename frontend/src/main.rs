mod api_handler;
mod components;
mod constants;
mod hooks;
mod pages;
mod routes;

use crate::pages::login::Login;
use api_handler::ApiHandler;
use common::UserInfo;
use components::Loading;
use routes::{switch, Route};
use yew::{function_component, html, use_effect_with_deps, use_state, ContextProvider};
use yew_router::{BrowserRouter, Switch};

#[function_component(App)]
fn app() -> Html {
    let api_handler = use_state(|| ApiHandler::new());
    let is_loading = use_state(|| false);
    let ah_clone = api_handler.clone();
    let is_loading_clone = is_loading.clone();

    use_effect_with_deps(
        move |_| {
            if *ah_clone.user_info.username != "".to_string() {
                let user_info = UserInfo {
                    api_key: None,
                    ..ah_clone.user_info.clone()
                };

                let body = serde_json::to_string(&user_info);
                if !body.is_err() {
                    let body = body.unwrap();

                    let is_loading_clone2 = is_loading_clone.clone();
                    let on_success = move |api_key: Option<String>| {
                        let user_info = UserInfo {
                            api_key,
                            ..ah_clone.user_info.clone()
                        };
                        ah_clone.set(ApiHandler { user_info });
                        is_loading_clone2.set(false);
                    };

                    is_loading_clone.set(true);
                    let is_loading_clone2 = is_loading_clone.clone();
                    let on_failure = move || {
                        is_loading_clone2.set(false);
                    };
                    ApiHandler::post("/login".to_string(), body, on_success, on_failure);
                }
            }
            || ()
        },
        (),
    );

    if *is_loading == true {
        return html! {<Loading/>};
    }

    html! {
        <>
            <ContextProvider<ApiHandler> context={(*api_handler).clone()}>
            {
                if (*api_handler).user_info.api_key == None {
                    html! {<Login api_handler={api_handler.clone()}/>}
                } else {
                    html!{
                        <BrowserRouter>
                            <Switch<Route> render={Switch::render(switch)} />
                        </BrowserRouter>
                    }
                }
              }
            </ContextProvider<ApiHandler>>
        </>

    }
}

fn main() {
    yew::start_app::<App>();
}
