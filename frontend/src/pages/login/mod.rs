use crate::api_handler::ApiHandler;
use common::UserInfo;
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

#[function_component(Login)]
pub fn login(props: &HomeProps) -> Html {
    let style_sheet = Style::new(include_str!("style.css")).expect("Css failed to load!");
    let login_username = use_mut_ref(|| "".to_string());
    let login_password = use_mut_ref(|| "".to_string());
    let signup_username = use_mut_ref(|| "".to_string());
    let signup_password = use_mut_ref(|| "".to_string());

    html! {
    <div class={{style_sheet}}>
        <div class="login">
            <h1>{"Already a member?"}</h1>
            <Form
                username_ref={login_username.clone()}
                password_ref={login_password.clone()}
                on_submit={get_handle_user(
                    "/login".to_string(),login_username.clone(),
                    login_password.clone(),props.api_handler.clone())
                } />

        </div>
        <div class="signup">
            <h2> {"Need an account?"}</h2>
            <Form
                username_ref={signup_username.clone()}
                password_ref={signup_password.clone()}
                on_submit={get_handle_user(
                    "/signup".to_string(),signup_username.clone(),
                    signup_password.clone(),props.api_handler.clone())

                } />

        </div>
    </div>
        }
}

fn create_onchange(mut_ref: Rc<RefCell<String>>) -> Callback<Event> {
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
            <input onchange={create_onchange(props.username_ref.clone())}type="text"  />
        </div>
        <div class="pair">
            <label for="lname">{"Password"}</label>
            <input onchange={create_onchange(props.password_ref.clone())} type="password" />
        </div>
        <input onclick={props.on_submit.clone()} type="submit" value="Log in"/>
    </form>


    }
}

fn set_local_storage(username: String, password: String) {
    let local_storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
    let _ = local_storage.set_item("username", &username);
    let _ = local_storage.set_item("password", &password);
}

fn get_handle_user(
    route: String,
    username: Rc<RefCell<String>>,
    password: Rc<RefCell<String>>,
    api_handler: UseStateHandle<ApiHandler>,
) -> Callback<MouseEvent> {
    Callback::from(move |event: MouseEvent| {
        event.prevent_default();
        let username = username.borrow().clone();
        let password = password.borrow().clone();
        let user_info = UserInfo {
            username: username.clone(),
            password: password.clone(),
            api_key: None,
        };

        let api_handler = api_handler.clone();

        let on_success = move |key: Option<String>| match key {
            Some(api_key) => {
                set_local_storage(username.clone(), password.clone());
                api_handler.set(ApiHandler {
                    user_info: UserInfo {
                        api_key: Some(api_key),
                        ..(*api_handler).user_info.clone()
                    },
                })
            }
            None => (),
        };

        match serde_json::to_string(&user_info) {
            Ok(body) => ApiHandler::post(route.clone(), body, on_success, || ()),
            Err(_) => (),
        }
    })
}
