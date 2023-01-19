mod embedded_resources;

use core::time::Duration;

use iced::{
    self, executor,
    widget::{self, button, column, image::Handle, row, Image},
    window, Application, Command, Element, Length, Theme,
};

use image::{DynamicImage, ImageBuffer, Rgba};

use rfd::FileDialog;

use mandellib::{render as sync_render, Frame, RenderParameters};

use embedded_resources::{ICON, RENDERING_IN_PROGRESS};

fn main() {
    let program_settings = iced::Settings {
        window: window::Settings {
            icon: Some(
                window::Icon::from_file_data(ICON, None)
                    .expect("embedded resources are correctly formatted images"),
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
    render_in_progress: bool,
    live_preview: bool,
    notifications: Vec<String>,
}

#[derive(Debug, Clone)]
enum Message {
    ReRenderPressed,
    RenderFinished(ImageBuffer<Rgba<u8>, Vec<u8>>),
    MaxItersUpdated(u32),
    PushNotification(String),
    PopNotification,
    LiveCheckboxToggled(bool),
    GrayscaleToggled(bool),
    SavePressed,
}
const INITIAL_X_RES: u32 = 1920;
const INITIAL_Y_RES: u32 = 1080;
const INITIAL_IMAG_DISTANCE: f64 = 8.0 / 3.0;
const INITIAL_SSAA_FACTOR: u8 = 3;
const INITIAL_MAX_ITERATIONS: u32 = 256;
const INITIAL_REAL_CENTER: f64 = -0.75;
const INITIAL_IMAG_CENTER: f64 = 0.0;
const PROGRAM_NAME: &str = "Mandelviewer";

async fn render(params: RenderParameters, frame: Frame) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    sync_render(params, frame, false).to_rgba8()
}

async fn background_timer(duration: Duration) {
    std::thread::sleep(duration)
}

impl MandelViewer {
    fn low_res(&self) -> RenderParameters {
        let mut low_res_params = self.params.clone();
        let aspect_ratio = f64::from(low_res_params.x_resolution.u32.get())
            / f64::from(low_res_params.y_resolution.u32.get());
        low_res_params.y_resolution = 480
            .try_into()
            .expect("480 fits in both a u32 and a usize and is nonzero");
        low_res_params.x_resolution = ((f64::from(low_res_params.y_resolution.u32.get())
            * aspect_ratio) as u32)
            .try_into()
            .expect("x-resolution should be valid when scaled down");
        low_res_params
    }
}

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

        (
            MandelViewer {
                image: None,
                params,
                view_region,
                render_in_progress: true,
                live_preview: false,
                notifications: Vec::new(),
            },
            Command::batch([
                window::maximize(true),
                Command::perform(render(params, view_region), Message::RenderFinished),
            ]),
        )
    }

    fn title(&self) -> String {
        PROGRAM_NAME.to_owned()
            + ": "
            + &self.view_region.center_real.to_string()
            + " + "
            + &self.view_region.center_imag.to_string()
            + "i"
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::MaxItersUpdated(max_iters) => {
                self.params.max_iterations = max_iters.try_into().expect("max_iters is not zero");
                if self.live_preview {
                    Command::perform(
                        render(self.low_res(), self.view_region),
                        Message::RenderFinished,
                    )
                } else {
                    Command::none()
                }
            }
            Message::ReRenderPressed => {
                self.render_in_progress = true;
                Command::perform(
                    render(self.params, self.view_region),
                    Message::RenderFinished,
                )
            }
            Message::PushNotification(e) => {
                self.notifications.push(e);
                Command::perform(background_timer(Duration::from_secs(5)), |_| {
                    Message::PopNotification
                })
            }
            Message::RenderFinished(buf) => {
                self.render_in_progress = false;
                self.image = Some(buf);
                Command::none()
            }
            Message::LiveCheckboxToggled(state) => {
                self.live_preview = state;
                Command::none()
            }
            Message::GrayscaleToggled(state) => {
                self.params.grayscale = state;
                if self.live_preview {
                    Command::perform(
                        render(self.low_res(), self.view_region),
                        Message::RenderFinished,
                    )
                } else {
                    Command::none()
                }
            }
            Message::SavePressed => {
                if let Some(ref img) = self.image {
                    match FileDialog::new()
                        .set_file_name("mandelbrot_set.png")
                        .add_filter("image", &["png", "jpg", "gif", "bmp", "tiff", "webp"])
                        .save_file()
                    {
                        Some(out_path) => {
                            if let Err(e) = DynamicImage::ImageRgba8(img.clone()).save(out_path) {
                                self.notifications.push(e.to_string());
                                Command::perform(background_timer(Duration::from_secs(5)), |_| {
                                    Message::PopNotification
                                })
                            } else {
                                Command::none()
                            }
                        }
                        None => Command::none(), // User cancelled save operation, do nothing.
                    }
                } else {
                    Command::none()
                }
            }
            Message::PopNotification => {
                self.notifications.drain(..=0);
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Self::Message> {
        let image_handle = match &self.image {
            Some(img) => Handle::from_pixels(img.width(), img.height(), img.clone().into_raw()),
            None => Handle::from_memory(RENDERING_IN_PROGRESS),
        };

        row![
            column![Image::new(image_handle).height(Length::Fill), {
                let notification_text: String = self
                    .notifications
                    .iter()
                    .cloned()
                    .map(|s| format!("{s}\n"))
                    .collect();
                widget::tooltip(
                    widget::text::Text::new(notification_text),
                    "",
                    widget::tooltip::Position::Bottom,
                )
            },]
            .width(Length::FillPortion(9)),
            column![
                widget::text::Text::new("Iterations"),
                row![
                    button("÷2").on_press(Message::MaxItersUpdated(
                        (self.params.max_iterations.get() / 2).max(1)
                    )),
                    widget::text_input::TextInput::new(
                        "Iterations",
                        &self.params.max_iterations.to_string(),
                        |max_iters| match max_iters.parse() {
                            Ok(mi) =>
                                if mi > 0 {
                                    Message::MaxItersUpdated(mi)
                                } else {
                                    Message::PushNotification(
                                        "the number of iterations must be at least 1".into(),
                                    )
                                },
                            Err(e) => {
                                Message::PushNotification(e.to_string())
                            }
                        }
                    ),
                    button("·2").on_press(Message::MaxItersUpdated(
                        self.params.max_iterations.get() * 2
                    )),
                ],
                widget::checkbox::Checkbox::new(self.params.grayscale, "Grayscale", |status| {
                    Message::GrayscaleToggled(status)
                }),
                {
                    let mut render_button = widget::Button::new("re-render view");
                    if !self.render_in_progress {
                        render_button = render_button.on_press(Message::ReRenderPressed)
                    }
                    render_button
                },
                widget::checkbox::Checkbox::new(self.live_preview, "Live preview", |status| {
                    Message::LiveCheckboxToggled(status)
                }),
                widget::vertical_space(Length::Fill),
                button("Save current view").on_press(Message::SavePressed),
            ]
            .width(Length::FillPortion(1)),
        ]
        .into()
    }
}
