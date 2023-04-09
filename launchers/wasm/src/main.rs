//use bevy::prelude::*;
use endless_sea::LAUNCHER_TITLE;
use stylist::{css, global_style, yew::styled_component};
use yew::{events::MouseEvent, prelude::*};

fn set_window_title(title: &str) {
    web_sys::window()
        .map(|w| w.document())
        .flatten()
        .expect("Unable to get DOM")
        .set_title(title);
}

fn set_global_css() {
    global_style! {
        r#"
        html {
            min-height: 100%;
            position: relative;
        }
        body {
            height: 100%;
            padding: 0;
            margin: 0;
        }
        "#
    }
    .expect("Unable to mount global style");
}

pub struct App;

impl Component for App {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        set_window_title(LAUNCHER_TITLE);
        set_global_css();

        Self
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let styles = css!(
            r#"
            position: absolute;
            overflow: hidden;
            width: 100%;
            height: 100%;
            "#
        );

        html! {
            <div class={styles}>
                <canvas id="bevy" oncontextmenu={|mouse_event: MouseEvent| mouse_event.prevent_default() }></canvas>
                <p>{"test"}</p>
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
