//use bevy::prelude::*;
use endless_sea::LAUNCHER_TITLE;
use yew::{events::MouseEvent, prelude::*};

fn set_window_title(title: &str) {
    web_sys::window()
        .map(|w| w.document())
        .flatten()
        .expect("Unable to get DOM")
        .set_title(title);
}

pub struct App;

impl Component for App {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        set_window_title(LAUNCHER_TITLE);

        Self
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div id="Bevy-Container">
                <canvas id="bevy" oncontextmenu={|mouse_event: MouseEvent| mouse_event.prevent_default() }></canvas>
            </div>
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, first_render: bool) {
        if first_render {
            let mut app = endless_sea::app();
            app.run();
        }
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
