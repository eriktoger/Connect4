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

#[function_component(Home)]
pub fn home(props: &HomeProps) -> Html {
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
                    login_password.clone(),
                    props.api_handler.clone())
                } />

        </div>
        <div class="signup">
            <h2> {"Need an account?"}</h2>
            <Form
                username_ref={signup_username.clone()}
                password_ref={signup_password.clone()}
                on_submit={get_handle_user(
                    "/signup".to_string(),signup_username.clone(),
                    signup_password.clone(),
                    props.api_handler.clone())
                } />

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
    let _ = local_storage.set_item("username", &user_info.username);
    let _ = local_storage.set_item("password", &user_info.password);
}

fn get_handle_user(
    route: String,
    log_name: Rc<RefCell<String>>,
    log_pw: Rc<RefCell<String>>,
    api_handler: UseStateHandle<ApiHandler>,
) -> Callback<MouseEvent> {
    Callback::from(move |event: MouseEvent| {
        event.prevent_default();
        let user_info = UserInfo {
            username: log_name.borrow().clone(),
            password: log_pw.borrow().clone(),
            api_key: None,
        };
        let api_handler_clone = api_handler.clone();
        let serialized = serde_json::to_string(&user_info).unwrap();

        let log_name = log_name.clone();
        let log_pw = log_pw.clone();
        let route = route.clone();

        //this wasm_bindgen is not needed
        wasm_bindgen_futures::spawn_local(async move {
            let api_handler_clone2 = api_handler_clone.clone();

            let action = move |key: Option<String>| match key {
                Some(key) => {
                    let user_info = UserInfo {
                        username: log_name.borrow().clone(),
                        password: log_pw.borrow().clone(),
                        api_key: Some(key),
                    };
                    set_local_storage(user_info.clone());
                    api_handler_clone2.set(ApiHandler { user_info });
                }
                None => (),
            };

            api_handler_clone.post(route, serialized, action);
        });
    })
}
