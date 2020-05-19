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
    Tick,
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
    fps: Fps,
}

impl UniverseModel {
    fn step(&mut self) {
        for _ in 0..self.n_steps {
            self.universe.step();
        }
        self.draw_game();
        self.fps.render();
        log!("Game Step!");
    }

    fn draw_game(&self) {
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document
            .get_element_by_id("game-of-life-canvas")
            .expect("there to be a canvas");
        let canvas: web_sys::HtmlCanvasElement = canvas
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .map_err(|_| ())
            .unwrap();

        let width = self.universe.width();
        let height = self.universe.height();
        canvas.set_height(((CELL_SIZE + 1) * height + 1) as u32);
        canvas.set_width(((CELL_SIZE + 1) * width + 1) as u32);

        let context = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap();

        log!("Draw!");

        self.draw_grid(&context);
        self.draw_cells(&context);
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
}

impl Component for UniverseModel {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let callback = link.callback(|_| {
            log!("tick callback!");
            Msg::Tick
        });
        let mut interval = yew::services::IntervalService::new();
        let handle = interval.spawn(std::time::Duration::from_millis(200), callback);

        log!("universe created!");

        UniverseModel {
            link,
            active: false,
            n_steps: 1,
            universe: gameoflife::Universe::new(128, 128),
            fps: Fps::default(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Random => {
                self.universe.randomize();
                log!("Random");
                self.draw_game();
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
                self.draw_game();
            }
            Msg::Stop => {
                self.active = false;
                log!("Stop");
            }
            Msg::ToggleCellule(idx) => {
                self.universe.toggle_cell(idx);
                self.draw_game();
            }
            Msg::Tick => {
                if self.active {
                    self.step();
                } else {
                    self.draw_game();
                }
                log!("Tick");
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
        html! {
            <section class="game-area">
                <div>{ self.fps.view() }</div>
                <canvas id="game-of-life-canvas"></canvas>
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
            self.draw_game();
        }
    }
}

pub struct Fps {
    frames: Vec<f32>,
    last_frame_time_stamp: f64,
    fps: f32,
    mean: f32,
    min: f32,
    max: f32,
}

impl Fps {
    fn render(&mut self) {
        let performance = web_sys::window().unwrap().performance().unwrap();
        let now = performance.now();
        let delta = now - self.last_frame_time_stamp;
        self.last_frame_time_stamp = now;

        let fps = 1f32 / delta as f32 * 1000f32;

        self.frames.push(fps);
        if self.frames.len() > 100 {
            self.frames.remove(0);
        }

        let mut min = std::f32::MAX;
        let mut max = std::f32::MIN;
        let mut sum = 0f32;
        for frame in &self.frames {
            sum += frame;
            min = min.min(*frame);
            max = max.max(*frame);
        }

        self.fps = fps;
        self.mean = sum / self.frames.len() as f32;
        self.min = min;
        self.max = max;
    }

    fn view(&self) -> Html {
        html! {
            <pre>
            {
                format!("Frames per Second:\n\
                        latest = {fps}\n\
                avg of last 100 = {mean}\n\
                min of last 100 = {min}\n\
                max of last 100 = {max}",
                fps=self.fps.round(),
                mean=self.mean.round(),
                min=self.min.round(),
                max=self.max.round()
                )
            }
            </pre>
        }
    }
}

impl Default for Fps {
    fn default() -> Fps {
        let performance = web_sys::window().unwrap().performance().unwrap();
        Fps {
            frames: vec![],
            last_frame_time_stamp: performance.now(),
            fps: 0f32,
            mean: 0f32,
            min: 0f32,
            max: 0f32,
        }
    }
}
