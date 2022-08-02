use crate::{api_handler::ApiHandler, constants::API_ROUTE};
use common::UserInfo;
use reqwasm::http::Request;
use std::{
    cell::RefCell,
    rc::{self, Rc},
};
use stylist::Style;
use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlInputElement, MouseEvent};
use yew::{function_component, html, use_mut_ref, Callback, Properties, UseStateHandle};

#[derive(Properties, PartialEq)]
pub struct HomeProps {
    pub api_handler: UseStateHandle<ApiHandler>,
}

#[function_component(Home)]
pub fn home(props: &HomeProps) -> Html {
    let style_sheet = Style::new(include_str!("style.css")).expect("Css failed to load!");

    let login_username = use_mut_ref(|| "".to_string());
    let login_password = use_mut_ref(|| "".to_string());
    let signup_username = use_mut_ref(|| "".to_string());
    let signup_password = use_mut_ref(|| "".to_string());

    let log_name = login_username.clone();
    let log_pw = login_password.clone();

    let api_handler_clone = props.api_handler.clone();
    let login = Callback::from(move |event: MouseEvent| {
        event.prevent_default();
        let user_info = UserInfo {
            username: log_name.borrow().clone(),
            password: log_pw.borrow().clone(),
        };
        let api_handler_clone = api_handler_clone.clone();
        let serialized = serde_json::to_string(&user_info).unwrap();

        let log_name = log_name.clone();
        let log_pw = log_pw.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let url = format!("{}{}", API_ROUTE, "/login");
            let api_handler_clone2 = api_handler_clone.clone();

            let action = move |key: Option<String>| match key {
                Some(key) => {
                    let user_info = UserInfo {
                        username: log_name.borrow().clone(),
                        password: log_pw.borrow().clone(),
                    };
                    set_local_storage(user_info.clone());
                    api_handler_clone2.set(ApiHandler {
                        user_info,
                        api_key: Some(key),
                    });
                }
                None => (),
            };

            api_handler_clone.post("/login".to_string(), serialized, action);
        });
    });

    let sign_name = signup_username.clone();
    let sign_pw = signup_password.clone();
    let signup = Callback::from(move |event: MouseEvent| {
        event.prevent_default();
        let user_info = UserInfo {
            username: sign_name.borrow().clone(),
            password: sign_pw.borrow().clone(),
        };
        let serialized = serde_json::to_string(&user_info).unwrap();
        wasm_bindgen_futures::spawn_local(async move {
            let url = format!("{}{}", API_ROUTE, "/signup");
            Request::post(&url).body(&serialized).send().await.unwrap();
            // if success save to context and go to lobby
        });
    });

    html! {
    <div class={{style_sheet}}>
        <div class="login">
            <h1>{"Already a member?"}</h1>
            <Form
                username_ref={login_username}
                password_ref={login_password}
                on_submit={login} />

        </div>
        <div class="signup">
            <h2> {"Need an account?"}</h2>
            <Form
                username_ref={signup_username}
                password_ref={signup_password}
                on_submit={signup} />
        </div>
    </div>
        }
}

fn create_cb(mut_ref: Rc<RefCell<String>>) -> Callback<Event> {
    Callback::from(move |event: Event| {
        event.prevent_default();
        let text = event
            .target()
            .unwrap()
            .unchecked_into::<HtmlInputElement>()
            .value();
        *mut_ref.borrow_mut() = text.clone();
    })
}

#[derive(Properties, PartialEq, Debug, Clone)]
struct FormProps {
    username_ref: rc::Rc<RefCell<String>>,
    password_ref: rc::Rc<RefCell<String>>,
    on_submit: Callback<MouseEvent>,
}

#[function_component(Form)]
fn form(props: &FormProps) -> Html {
    html! {
        <form  class="form">
        <div class="pair">
            <label>{"User name"}</label>
            <input onchange={create_cb(props.username_ref.clone())}type="text"  />
        </div>
        <div class="pair">
            <label for="lname">{"Password"}</label>
            <input onchange={create_cb(props.password_ref.clone())} type="password" />
        </div>
        <input onclick={props.on_submit.clone()} type="submit" value="Log in"/>
    </form>


    }
}

fn set_local_storage(user_info: UserInfo) {
    let local_storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
    let username = local_storage.set_item("username", &user_info.username);
    let password = local_storage.set_item("password", &user_info.password);
}
