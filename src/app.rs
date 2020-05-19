use crate::fps;
use crate::gameoflife;

use yew::prelude::*;

use wasm_bindgen::JsCast;

pub struct App {}

pub enum Msg {
    Random,
    Start,
    Step,
    Reset,
    Stop,
    ToggleCellule(usize),
    Tick(f64),
}

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
                      { "Game of Life - in rust - in honer of Conway" }
                    </strong>
                </footer>
            </div>
        }
    }
}

const CELL_SIZE: usize = 5; // px
const GRID_COLOR: &str = "#CCCCCC";
const DEAD_COLOR: &str = "#FFFFFF";
const ALIVE_COLOR: &str = "#000000";

pub struct UniverseModel {
    link: ComponentLink<Self>,
    active: bool,
    n_steps: usize,
    universe: gameoflife::Universe,
    fps: fps::Fps,
    fps_html: String,
    canvas_node_ref: NodeRef,
    canvas: Option<web_sys::HtmlCanvasElement>,
    ctx: Option<web_sys::CanvasRenderingContext2d>,
    render_handle: Option<Box<dyn yew::services::Task>>,
}

impl UniverseModel {
    fn step(&mut self) {
        for _ in 0..self.n_steps {
            self.universe.step();
        }
    }

    fn draw_game(&mut self) {
        let canvas = self.canvas.as_ref().expect("canvas not initialised!");

        let width = self.universe.width();
        let height = self.universe.height();

        canvas.set_height(((CELL_SIZE + 1) * height + 1) as u32);
        canvas.set_width(((CELL_SIZE + 1) * width + 1) as u32);

        let ctx = self.ctx.as_ref().expect("canvas context not initialise!");

        self.draw_grid(&ctx);
        self.draw_cells(&ctx);
        self.fps.render();
        self.fps_html = self.fps.get_html();
    }

    fn draw_grid(&self, ctx: &web_sys::CanvasRenderingContext2d) {
        ctx.begin_path();
        ctx.set_stroke_style(&GRID_COLOR.into());

        let width = self.universe.width();
        let height = self.universe.height();

        // Vertical lines.
        for i in 0..=width {
            ctx.move_to((i * (CELL_SIZE + 1) + 1) as f64, 0 as f64);
            ctx.line_to(
                (i * (CELL_SIZE + 1) + 1) as f64,
                ((CELL_SIZE + 1) * height + 1) as f64,
            );
        }

        // Horizontal lines.
        for j in 0..=height {
            ctx.move_to(0 as f64, (j * (CELL_SIZE + 1) + 1) as f64);
            ctx.line_to(
                ((CELL_SIZE + 1) * width + 1) as f64,
                (j * (CELL_SIZE + 1) + 1) as f64,
            );
        }

        ctx.stroke();
    }

    fn get_index(&self, row: usize, column: usize) -> usize {
        row * self.universe.width() + column
    }

    fn draw_cells(&self, ctx: &web_sys::CanvasRenderingContext2d) {
        let cells = self.universe.get_cells();
        let width = self.universe.width();
        let height = self.universe.height();

        ctx.begin_path();

        // alive cells
        ctx.set_fill_style(&ALIVE_COLOR.into());
        for row in 0..height {
            for col in 0..width {
                let idx = self.get_index(row, col);
                if !cells[idx] {
                    continue;
                }

                ctx.fill_rect(
                    (col * (CELL_SIZE + 1) + 1) as f64,
                    (row * (CELL_SIZE + 1) + 1) as f64,
                    CELL_SIZE as f64,
                    CELL_SIZE as f64,
                )
            }
        }

        // dead cells
        ctx.set_fill_style(&DEAD_COLOR.into());
        for row in 0..height {
            for col in 0..width {
                let idx = self.get_index(row, col);
                if cells[idx] {
                    continue;
                }

                ctx.fill_rect(
                    (col * (CELL_SIZE + 1) + 1) as f64,
                    (row * (CELL_SIZE + 1) + 1) as f64,
                    CELL_SIZE as f64,
                    CELL_SIZE as f64,
                )
            }
        }

        ctx.stroke();
    }

    fn render_loop(&mut self) {
        self.draw_game();

        let render_frame = self.link.callback(Msg::Tick);
        let handle = yew::services::RenderService::new().request_animation_frame(render_frame);

        // A reference to the new handle must be retained for the next render to run.
        self.render_handle = Some(Box::new(handle));
    }
}

impl Component for UniverseModel {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        log!("universe created!");

        UniverseModel {
            link,
            active: false,
            n_steps: 1,
            universe: gameoflife::Universe::new(128, 128),
            fps: fps::Fps::default(),
            fps_html: String::default(),
            canvas_node_ref: NodeRef::default(),
            canvas: None,
            ctx: None,
            render_handle: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Random => {
                self.universe.randomize();
                log!("Random");
            }
            Msg::Start => {
                if !self.active {
                    self.active = true;
                    log!("Start");
                }
            }
            Msg::Step => {
                self.step();
            }
            Msg::Reset => {
                self.universe.reset();
                log!("Reset");
            }
            Msg::Stop => {
                self.active = false;
                log!("Stop");
            }
            Msg::ToggleCellule(idx) => {
                self.universe.toggle_cell(idx);
            }
            Msg::Tick(_) => {
                if self.active {
                    self.step();
                }
                self.render_loop();
            }
        }
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        // Should only return "true" if new properties are different to
        // previously received properties.
        // This component has no properties so we will always return "false".
        false
    }

    fn view(&self) -> Html {
        log!("game reflowed");
        html! {
            <section class="game-area">
                <div> <fps::FpsModel fps_html=self.fps_html.clone() /></div>
                <canvas ref=self.canvas_node_ref.clone()></canvas>
                <div class="game-buttons">
                    <button class="game-button" onclick=self.link.callback(|_| Msg::Random)>{ "Random" }</button>
                    <button class="game-button" onclick=self.link.callback(|_| Msg::Step)>{ "Step" }</button>
                    <button class="game-button" onclick=self.link.callback(|_| Msg::Start)>{ "Start" }</button>
                    <button class="game-button" onclick=self.link.callback(|_| Msg::Stop)>{ "Stop" }</button>
                    <button class="game-button" onclick=self.link.callback(|_| Msg::Reset)>{ "Reset" }</button>
                </div>
            </section>
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            if let Some(canvas) = self.canvas_node_ref.cast::<web_sys::HtmlCanvasElement>() {
                let context = canvas
                    .get_context("2d")
                    .unwrap()
                    .unwrap()
                    .dyn_into::<web_sys::CanvasRenderingContext2d>()
                    .unwrap();

                self.canvas = Some(canvas);
                self.ctx = Some(context);
            }

            self.render_loop();
        }
    }
}
