use crate::api_handler::ApiHandler;
use yew::use_context;

pub fn use_api_handler() -> ApiHandler {
    use_context().expect("Api handler context missing")
}
