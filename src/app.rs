// main app container code
use crate::game::UniverseModel;
use yew::prelude::*;

pub struct App {}

pub enum Msg {}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self {
        log!("App create!");
        App {}
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        // Should only return "true" if new properties are different to
        // previously received properties.
        // This component has no properties so we will always return "false".
        false
    }

    fn view(&self) -> Html {
        html! {
            <div class="app-container">
                <section class="game-container">
                    <header class="app-header">
                        <h1 class="app-title">{ "Game of Life" }</h1>
                    </header>
                </section>
                <UniverseModel />
                <footer class="app-footer">
                    <strong class="footer-text">
                      { "Game of Life - In rust - In honor of " } <a href="https://www.conwaylife.com/wiki/John_Conway" >{" John Conway " }</a>
                    </strong>
                </footer>
            </div>
        }
    }
}
