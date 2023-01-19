mod embedded_resources;

use iced::{
    self, executor,
    widget::{self, button, column, image::Handle, row, Image},
    window, Application, Command, Element, Length, Theme,
};

use image::{ImageBuffer, Rgba};

use mandellib::{render, Frame, RenderParameters};

use embedded_resources::ICON;

fn main() {
    let program_settings = iced::Settings {
        window: window::Settings {
            icon: Some(
                window::Icon::from_file_data(ICON, None)
                    .expect("embedded resources are present and correct"),
            ),
            ..Default::default()
        },
        ..Default::default()
    };

    MandelViewer::run(program_settings).unwrap();
}

struct MandelViewer {
    image: Option<ImageBuffer<Rgba<u8>, Vec<u8>>>,
    params: RenderParameters,
    view_region: Frame,
    window_title: String,
}

#[derive(Debug, Clone)]
enum Message {
    ReRenderPressed,
    MaxItersUpdated(u32),
    InputParseFail(String),
}
const INITIAL_X_RES: u32 = 1920;
const INITIAL_Y_RES: u32 = 1080;
const INITIAL_IMAG_DISTANCE: f64 = 8.0 / 3.0;
const INITIAL_SSAA_FACTOR: u8 = 3;
const INITIAL_MAX_ITERATIONS: u32 = 255;
const INITIAL_REAL_CENTER: f64 = -0.75;
const INITIAL_IMAG_CENTER: f64 = 0.0;
const PROGRAM_NAME: &str = "Mandelviewer";

impl Application for MandelViewer {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();
    type Theme = Theme;

    fn new(_flags: ()) -> (MandelViewer, Command<Self::Message>) {
        let params = RenderParameters::new(
            INITIAL_X_RES.try_into().unwrap(),
            INITIAL_Y_RES.try_into().unwrap(),
            INITIAL_MAX_ITERATIONS.try_into().unwrap(),
            INITIAL_SSAA_FACTOR.try_into().unwrap(),
            false,
        )
        .unwrap();
        let view_region = Frame::new(
            INITIAL_REAL_CENTER,
            INITIAL_IMAG_CENTER,
            f64::from(INITIAL_X_RES) / f64::from(INITIAL_Y_RES) * INITIAL_IMAG_DISTANCE,
            INITIAL_IMAG_DISTANCE,
        );

        let image = render(params, view_region, false).into_rgba8();

        (
            MandelViewer {
                image: Some(image),
                params,
                view_region,
                window_title: PROGRAM_NAME.into(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        self.window_title.clone()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::MaxItersUpdated(max_iters) => {
                self.params.max_iterations = max_iters.try_into().expect("max_iters is not zero");
            }
            Message::ReRenderPressed => {
                self.image = Some(render(self.params, self.view_region, false).into_rgba8())
            }
            Message::InputParseFail(e) => {
                eprintln!("{e}")
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        let image_handle = match &self.image {
            Some(img) => Handle::from_pixels(img.width(), img.height(), img.clone().into_raw()),
            // If there is no rendered image, show the program icon.
            None => Handle::from_memory(ICON),
        };

        row![
            Image::new(image_handle)
                .width(Length::Fill)
                .height(Length::Fill),
            column![
                widget::text_input::TextInput::new(
                    "Max iterations",
                    &self.params.max_iterations.to_string(),
                    |max_iters| match max_iters.parse() {
                        Ok(mi) => Message::MaxItersUpdated(mi),
                        Err(e) => {
                            Message::InputParseFail(e.to_string())
                        }
                    }
                )
                .width(Length::Units(100)),
                widget::slider::Slider::new(1..=u32::MAX, INITIAL_MAX_ITERATIONS, |max_iters| {
                    Message::MaxItersUpdated(max_iters)
                })
                .width(Length::Units(100)),
                button("re-render view").on_press(Message::ReRenderPressed),
            ]
        ]
        .into()
    }
}
