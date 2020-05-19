use yew::prelude::*;

pub struct Fps {
    frames: Vec<f64>,
    last_frame_time_stamp: f64,
    fps: f64,
    mean: f64,
    min: f64,
    max: f64,
}

impl Fps {
    pub fn render(&mut self) {
        let performance = web_sys::window().unwrap().performance().unwrap();
        let now = performance.now();
        let delta = now - self.last_frame_time_stamp;
        self.last_frame_time_stamp = now;

        let fps = 1f64 / delta as f64 * 1000f64;

        self.frames.push(fps);
        if self.frames.len() > 100 {
            self.frames.remove(0);
        }

        let mut min = std::f64::MAX;
        let mut max = std::f64::MIN;
        let mut sum = 0f64;
        for frame in &self.frames {
            sum += frame;
            min = min.min(*frame);
            max = max.max(*frame);
        }

        self.fps = fps;
        self.mean = sum / self.frames.len() as f64;
        self.min = min;
        self.max = max;
    }

    pub fn get_html(&self) -> String {
        format!(
            "Frames per Second:\n\
                latest = {fps}\n\
            avg of last 100 = {mean}\n\
            min of last 100 = {min}\n\
            max of last 100 = {max}",
            fps = self.fps.round(),
            mean = self.mean.round(),
            min = self.min.round(),
            max = self.max.round()
        )
    }
}

impl Default for Fps {
    fn default() -> Fps {
        let performance = web_sys::window().unwrap().performance().unwrap();
        Fps {
            frames: vec![],
            last_frame_time_stamp: performance.now(),
            fps: 0f64,
            mean: 0f64,
            min: 0f64,
            max: 0f64,
        }
    }
}

#[derive(Properties, Clone, PartialEq)]
pub struct FpsModelProps {
    #[prop_or_default]
    pub fps_html: String,
}

pub struct FpsModel {
    props: FpsModelProps,
}

impl Component for FpsModel {
    type Message = ();
    type Properties = FpsModelProps;

    fn create(props: Self::Properties, _: ComponentLink<Self>) -> Self {
        FpsModel { props }
    }

    fn update(&mut self, _: Self::Message) -> ShouldRender {
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }

    fn view(&self) -> Html {
        html! {
            <pre>
             { self.props.fps_html.clone() }
            </pre>
        }
    }
}
