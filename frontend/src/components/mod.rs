use stylist::Style;
use yew::{function_component, html, Children, Properties};

#[derive(Properties, PartialEq)]
pub struct GameRoomProps {
    pub path: String,
    pub children: Children,
}

#[function_component(Container)]
pub fn container(props: &GameRoomProps) -> Html {
    let common_style_sheet = Style::new(include_str!("style.css")).expect("Css failed to load");
    let style_sheet = Style::new(props.path.clone())
        .expect(format!("Css failed to load: {}", props.path).as_str());

    html! { <main class={common_style_sheet}>
                <div class="outer-container">
                    <div class="inner-container">
                        <div class={style_sheet}>
                            { props.children.clone() }
                        </div>
                    </div>
                </div>
            </main>
    }
}

#[function_component(Loading)]
pub fn loading() -> Html {
    let common_style_sheet = Style::new(include_str!("style.css")).expect("Css failed to load");

    html! {
    <main class={common_style_sheet}>
        <div class="outer-container">
            <div class="inner-container loading-container" >
                <span> {"Loading..."}</span>
                <span> {"Please be patient."} </span>
                <span> {"The backend is hosted on a free tier."} </span>
            </div>
        </div>
    </main>}
}
