// Game container code

use crate::fps;
use crate::universe::Universe;

use yew::prelude::*;

//
use wasm_bindgen::JsCast;

pub struct KeysPressed {
    ctrl: bool,
    shift: bool,
    alt: bool,
    meta: bool,
}

impl std::fmt::Display for KeysPressed {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "[ctrl: {}, shift: {} alt: {}, meta: {}]",
            self.ctrl, self.shift, self.alt, self.meta
        )
    }
}

impl KeysPressed {
    pub fn only_ctrl(&self) -> bool {
        self.ctrl && !self.shift && !self.alt && !self.meta
    }
    pub fn only_shift(&self) -> bool {
        !self.ctrl && self.shift && !self.alt && !self.meta
    }
    #[allow(dead_code)]
    pub fn only_alt(&self) -> bool {
        !self.ctrl && !self.shift && self.alt && !self.meta
    }
    #[allow(dead_code)]
    pub fn only_ctrl_shift(&self) -> bool {
        self.ctrl && self.shift && !self.alt && !self.meta
    }
    #[allow(dead_code)]
    pub fn only_ctrl_alt(&self) -> bool {
        self.ctrl && !self.shift && self.alt && !self.meta
    }
}

pub enum Msg {
    Random,
    Step,
    Reset,
    Click(i32, i32, KeysPressed),
    Tick(f64),
    TickToggle,
    StepsPerTick(usize),
}

const CELL_SIZE: usize = 5; // px
const GRID_COLOR: &str = "#CCCCCC";
const DEAD_COLOR: &str = "#FFFFFF";
const ALIVE_COLOR: &str = "#000000";

pub struct UniverseModel {
    link: ComponentLink<Self>,
    active: bool,
    n_steps: usize,
    universe: Universe,
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

    fn get_index(&self, row: usize, col: usize) -> usize {
        self.universe.get_morton(row, col)
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
        let handle = yew::services::RenderService::request_animation_frame(render_frame);

        // A reference to the new handle must be retained for the next render to run.
        self.render_handle = Some(Box::new(handle));
    }

    fn get_row_col_client_cords(&self, x: i32, y: i32) -> (usize, usize) {
        let canvas = self.canvas.as_ref().expect("canvas not initialised!");
        let bounding_rect = canvas.get_bounding_client_rect();

        let scale_x = canvas.width() as f64 / bounding_rect.width();
        let scale_y = canvas.height() as f64 / bounding_rect.height();

        let canvas_left = (x as f64 - bounding_rect.left()) * scale_x;
        let canvas_top = (y as f64 - bounding_rect.top()) * scale_y;
        let row = (canvas_top / (CELL_SIZE + 1) as f64)
            .floor()
            .min((self.universe.height() - 1) as f64);
        let col = (canvas_left / (CELL_SIZE + 1) as f64)
            .floor()
            .min((self.universe.width() - 1) as f64);

        (row as usize, col as usize)
    }

    fn process_context_click(&mut self, x: i32, y: i32, keys: KeysPressed) {
        let (row, col) = self.get_row_col_client_cords(x, y);
        if keys.only_ctrl() {
            self.universe.set_flyer(row, col);
        } else if keys.only_shift() {
            self.universe.set_pulsar(row, col);
        } else {
            self.universe.toggle_cell(row, col);
        }
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
            universe: Universe::new(256, 256),
            fps: fps::Fps::default(),
            fps_html: String::default(),
            canvas_node_ref: NodeRef::default(),
            canvas: None,
            ctx: None,
            render_handle: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        match msg {
            Msg::Random => {
                self.universe.randomize();
                log!("Random");
                false
            }
            Msg::Step => {
                self.step();
                false
            }
            Msg::Reset => {
                self.universe.reset();
                log!("Reset");
                false
            }
            Msg::TickToggle => {
                if !self.active {
                    self.active = true;
                    log!("Start");
                } else {
                    self.active = false;
                    log!("Stop");
                }
                true
            }
            Msg::Click(x, y, keys) => {
                self.process_context_click(x, y, keys);
                false
            }
            Msg::Tick(_) => {
                if self.active {
                    self.step();
                }
                self.render_loop();
                true
            }
            Msg::StepsPerTick(n) => {
                self.n_steps = n;
                log!("Steps per tick is now: {}", n);
                true
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> bool {
        // Should only return "true" if new properties are different to
        // previously received properties.
        // This component has no properties so we will always return "false".
        false
    }

    fn view(&self) -> Html {
        let n_steps = self.n_steps;
        html! {
            <section class="game-area">
                <div> <fps::FpsModel fps_html={self.fps_html.clone()} /></div>
                <canvas ref=self.canvas_node_ref.clone()
                    onclick=self.link.callback(|e: web_sys::MouseEvent|{
                        let keys = KeysPressed {
                            ctrl: e.ctrl_key(),
                            shift: e.shift_key(),
                            alt: e.alt_key(),
                            meta: e.meta_key()
                        };
                        log!("{}:{} -> {}", e.client_x(), e.client_y(), keys);
                        Msg::Click(e.client_x(), e.client_y(), keys)
                    })></canvas>
                <div class="game-buttons">
                    <button class="game-button" onclick=self.link.callback(|_| Msg::TickToggle)> {if self.active {"⏸"} else {"▶"}}</button>
                    <button class="game-button" onclick=self.link.callback(|_| Msg::Random)>{ "Randomize" }</button>
                    <button class="game-button" onclick=self.link.callback(|_| Msg::Step)>{ "Step" }</button>
                    <button class="game-button" onclick=self.link.callback(|_| Msg::Reset)>{ "Clear" }</button>
                    <div>
                        <label> { format!("Ticks per Frame: {}", n_steps) } </label>
                        <input type="range" id="ticks-per-frame" min="1" max="20" value="1" onchange=self.link.callback(move |value| {
                                let mut n = n_steps;
                                if let yew::events::ChangeData::Value(str_n) = value {
                                    let result = str_n.parse::<usize>();
                                    if let Ok(i) = result {
                                        n = i;
                                    }
                                }
                                Msg::StepsPerTick(n)
                        }) />
                    </div>
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
