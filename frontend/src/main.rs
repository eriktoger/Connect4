mod api_handler;
mod constants;
mod pages;
mod routes;
use crate::pages::home::Home;
use api_handler::ApiHandler;
use common::UserInfo;
use routes::{switch, Route};
use yew::{function_component, html, use_effect_with_deps, use_state, ContextProvider};
use yew_router::{BrowserRouter, Switch};

#[function_component(App)]
fn app() -> Html {
    let api_handler = use_state(|| ApiHandler::new());
    let is_loading = use_state(|| true);
    let ah_clone = api_handler.clone();
    let is_loading_clone = is_loading.clone();

    use_effect_with_deps(
        move |_| {
            let (username, password) = check_local_storage();
            let user_info = UserInfo {
                username: username.clone(),
                password: password.clone(),
                api_key: None,
            };
            let serialized = serde_json::to_string(&user_info).unwrap();

            let ah_clone2 = ah_clone.clone();
            let is_loading_clone2 = is_loading_clone.clone();
            let action = move |key: Option<String>| {
                let user_info = UserInfo {
                    username: username.clone(),
                    password: password.clone(),
                    api_key: key,
                };
                ah_clone2.set(ApiHandler {
                    user_info: user_info.clone(),
                });
                is_loading_clone2.set(false);
            };

            is_loading_clone.set(true);
            ApiHandler::post("/login".to_string(), serialized, action);

            || ()
        },
        (),
    );

    if *is_loading == true {
        return html! {<div>{"Loading"}</div>};
    }

    html! {
            <ContextProvider<ApiHandler> context={(*api_handler).clone()}>
            {
                if (*api_handler).user_info.api_key == None {
                    html! {<Home api_handler={api_handler.clone()}/>}
                } else {
                    html!{
                        <BrowserRouter>
                            <Switch<Route> render={Switch::render(switch)} />
                        </BrowserRouter>
                    }
                }
              }
            </ContextProvider<ApiHandler>>

    }
}

fn main() {
    yew::start_app::<App>();
}

fn check_local_storage() -> (String, String) {
    let local_storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
    let username = local_storage
        .get_item("username")
        .unwrap()
        .unwrap_or_default();
    let password = local_storage
        .get_item("password")
        .unwrap()
        .unwrap_or_default();
    (username, password)
}
