use core::{
    num::{NonZeroU32, NonZeroU8},
    time::Duration,
};
use std::num::TryFromIntError;

mod embedded_resources;
use embedded_resources::{ICON, RENDERING_IN_PROGRESS};
use mandellib::{render as sync_render, Frame, RenderParameters};

use iced::{
    self, executor,
    widget::{
        button::Button,
        checkbox::Checkbox,
        column,
        image::{Handle, Viewer},
        row,
        text::Text,
        text_input::TextInput,
        tooltip::{Position, Tooltip},
        Slider, Space,
    },
    window, Application, Command, Element, Length, Theme,
};
use image::{DynamicImage, ImageFormat};
use rfd::FileDialog;

// Safety: This results in undefined behavior if any of the values are zero, but these are not zero.
const PREVIEW_RES: NonZeroU32 = unsafe { NonZeroU32::new_unchecked(480) };
const INITIAL_X_RES: NonZeroU32 = unsafe { NonZeroU32::new_unchecked(1920) };
const INITIAL_Y_RES: NonZeroU32 = unsafe { NonZeroU32::new_unchecked(1080) };
const INITIAL_SSAA_FACTOR: NonZeroU8 = unsafe { NonZeroU8::new_unchecked(3) };
const INITIAL_MAX_ITERATIONS: NonZeroU32 = unsafe { NonZeroU32::new_unchecked(256) };

const ASPECT_RATIO: f64 = 16.0 / 9.0;
const INITIAL_IMAG_DISTANCE: f64 = 8.0 / 3.0;
const INITIAL_REAL_CENTER: f64 = -0.75;
const INITIAL_IMAG_CENTER: f64 = 0.0;
const PROGRAM_NAME: &str = "Mandelviewer";
const NOTIFICATION_DURATION: u64 = 5;

fn main() {
    let program_settings = iced::Settings {
        window: window::Settings {
            icon: Some(
                window::Icon::from_file_data(ICON, Some(ImageFormat::Png))
                    .expect("embedded resources are correctly formatted images"),
            ),
            ..Default::default()
        },
        ..Default::default()
    };

    MandelViewer::run(program_settings).unwrap();
}

async fn render(params: RenderParameters, frame: Frame, verbose: bool) -> DynamicImage {
    sync_render(params, frame, verbose)
}

struct UIValues {
    slider_ssaa_factor: NonZeroU8,
    do_ssaa: bool,
    live_preview: bool,
}

struct MandelViewer {
    image: Option<DynamicImage>,
    params: RenderParameters,
    view_region: Frame,
    render_in_progress: bool,
    notifications: Vec<String>,
    ui_values: UIValues,
}

#[derive(Debug, Clone)]
enum Message {
    ReRenderPressed,
    RenderFinished(DynamicImage),
    MaxItersUpdated(NonZeroU32),
    PushNotification(String),
    PopNotification,
    LiveCheckboxToggled(bool),
    GrayscaleToggled(bool),
    SavePressed,
    VerticalResolutionUpdated(NonZeroU32),
    SuperSamplingToggled(bool),
    SuperSamplingUpdated(NonZeroU8),
}

async fn background_timer(duration: Duration) {
    std::thread::sleep(duration)
}

impl MandelViewer {
    fn change_resolution(&self, y_res: NonZeroU32) -> Result<RenderParameters, TryFromIntError> {
        let mut new_params = self.params;
        new_params.y_resolution = y_res.try_into()?;
        new_params.x_resolution = ((f64::from(y_res.get()) * ASPECT_RATIO) as u32).try_into()?;
        Ok(new_params)
    }

    fn push_notification(&mut self, text: String) -> Command<<Self as Application>::Message> {
        self.notifications.push(text);
        Command::perform(
            background_timer(Duration::from_secs(NOTIFICATION_DURATION)),
            |_| Message::PopNotification,
        )
    }
}

impl Application for MandelViewer {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();
    type Theme = Theme;

    fn new(_flags: ()) -> (MandelViewer, Command<Self::Message>) {
        let params = RenderParameters::try_new(
            INITIAL_X_RES,
            INITIAL_Y_RES,
            INITIAL_MAX_ITERATIONS,
            INITIAL_SSAA_FACTOR,
            false,
        )
        .unwrap();
        let view_region = Frame::new(
            INITIAL_REAL_CENTER,
            INITIAL_IMAG_CENTER,
            f64::from(INITIAL_X_RES.get()) / f64::from(INITIAL_Y_RES.get()) * INITIAL_IMAG_DISTANCE,
            INITIAL_IMAG_DISTANCE,
        );

        (
            MandelViewer {
                image: None,
                params,
                view_region,
                render_in_progress: true,
                notifications: Vec::new(),
                ui_values: UIValues {
                    slider_ssaa_factor: INITIAL_SSAA_FACTOR,
                    do_ssaa: true,
                    live_preview: false,
                },
            },
            Command::batch([
                window::maximize(true),
                Command::perform(render(params, view_region, false), Message::RenderFinished),
            ]),
        )
    }

    fn title(&self) -> String {
        PROGRAM_NAME.to_owned()
        // + ": "
        // + &self.view_region.center_real.to_string()
        // + " + "
        // + &self.view_region.center_imag.to_string()
        // + "i"
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::MaxItersUpdated(max_iters) => {
                self.params.max_iterations = max_iters;
                if self.ui_values.live_preview {
                    Command::perform(
                        render(
                            self.change_resolution(PREVIEW_RES)
                                .expect("PREVIEW_RES is a valid resolution"),
                            self.view_region,
                            false,
                        ),
                        Message::RenderFinished,
                    )
                } else {
                    Command::none()
                }
            }
            Message::ReRenderPressed => {
                self.render_in_progress = true;
                // Clear viewer to save memory
                self.image = None;
                Command::perform(
                    render(self.params, self.view_region, false),
                    Message::RenderFinished,
                )
            }
            Message::PushNotification(e) => self.push_notification(e),
            Message::RenderFinished(buf) => {
                self.render_in_progress = false;
                self.image = Some(buf);
                Command::none()
            }
            Message::LiveCheckboxToggled(state) => {
                self.ui_values.live_preview = state;
                Command::none()
            }
            Message::GrayscaleToggled(state) => {
                self.params.grayscale = state;
                if self.ui_values.live_preview {
                    Command::perform(
                        render(
                            self.change_resolution(PREVIEW_RES)
                                .expect("PREVIEW_RES is a valid resolution"),
                            self.view_region,
                            false,
                        ),
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
                            if let Err(e) = img.save(out_path) {
                                self.push_notification(e.to_string())
                            } else {
                                self.push_notification("save operation successful".into())
                            }
                        }
                        None => self.push_notification("save operation cancelled".into()),
                    }
                } else {
                    self.push_notification("no image to save".into())
                }
            }
            Message::PopNotification => {
                self.notifications.drain(..=0);
                Command::none()
            }
            Message::VerticalResolutionUpdated(y_res) => match self.change_resolution(y_res) {
                Ok(params) => {
                    if params.x_resolution.u32.get() * params.y_resolution.u32.get() * 4
                        <= 1000000000
                    {
                        self.params = params;
                        if self.ui_values.live_preview {
                            Command::perform(
                                render(self.params, self.view_region, false),
                                Message::RenderFinished,
                            )
                        } else {
                            Command::none()
                        }
                    } else {
                        self.push_notification("the resolution is too large".into())
                    }
                }
                Err(e) => self.push_notification(e.to_string()),
            },
            Message::SuperSamplingToggled(do_ssaa) => {
                self.ui_values.do_ssaa = do_ssaa;
                if !self.ui_values.do_ssaa {
                    self.params.sqrt_samples_per_pixel = 1.try_into().expect("1 is not zero");
                } else {
                    self.params.sqrt_samples_per_pixel = self.ui_values.slider_ssaa_factor;
                };

                if self.ui_values.live_preview {
                    Command::perform(
                        render(
                            self.change_resolution(PREVIEW_RES)
                                .expect("PREVIEW_RES is a valid resolution"),
                            self.view_region,
                            false,
                        ),
                        Message::RenderFinished,
                    )
                } else {
                    Command::none()
                }
            }
            Message::SuperSamplingUpdated(ssaa_factor) => {
                self.ui_values.slider_ssaa_factor = ssaa_factor;
                if self.ui_values.live_preview && self.ui_values.do_ssaa {
                    self.params.sqrt_samples_per_pixel = self.ui_values.slider_ssaa_factor;
                    Command::perform(
                        render(
                            self.change_resolution(PREVIEW_RES)
                                .expect("PREVIEW_RES is a valid resolution"),
                            self.view_region,
                            false,
                        ),
                        Message::RenderFinished,
                    )
                } else {
                    Command::none()
                }
            }
        }
    }

    fn view(&self) -> Element<Self::Message> {
        row![
            // An image viewer with an expanding notification field above it.
            column![
                Text::new(
                    self.notifications
                        .iter()
                        .rev()
                        .cloned()
                        .map(|s| format!("{s}\n"))
                        .collect::<String>()
                ),
                Viewer::new(match &self.image {
                    Some(img) =>
                        Handle::from_pixels(img.width(), img.height(), img.to_rgba8().into_raw()),
                    None =>
                        if self.render_in_progress {
                            Handle::from_memory(RENDERING_IN_PROGRESS)
                        } else {
                            Handle::from_memory(ICON)
                        },
                })
                .height(Length::Fill),
            ]
            .width(Length::FillPortion(8)),
            Space::new(Length::Units(20), Length::Shrink),
            // A column with rendering settings
            column![
                // A text input field for the y-resolution with buttons on either side to halve or double it.
                Text::new("Vertical resolution"),
                row![
                    Button::new("÷2").on_press(Message::VerticalResolutionUpdated(
                        self.params
                            .y_resolution
                            .u32
                            .get()
                            .saturating_div(2)
                            .max(1)
                            .try_into()
                            .expect("never zero")
                    )),
                    TextInput::new(
                        "Vertical resolution",
                        &self.params.y_resolution.u32.get().to_string(),
                        |yres| match yres.parse() {
                            Ok(mi) => {
                                Message::VerticalResolutionUpdated(mi)
                            }
                            Err(e) => Message::PushNotification(e.to_string()),
                        }
                    )
                    .on_submit(Message::ReRenderPressed),
                    Button::new("·2").on_press(Message::VerticalResolutionUpdated(
                        (self.params.y_resolution.u32.get().saturating_mul(2))
                            .try_into()
                            .expect("doubling a number never gives zero")
                    ))
                ],
                // A text input field for the number of iterations with buttons on either side to halve or double it.
                Text::new("Iterations"),
                row![
                    Button::new("÷2").on_press(Message::MaxItersUpdated(
                        self.params
                            .max_iterations
                            .get()
                            .saturating_div(2)
                            .max(1)
                            .try_into()
                            .expect("never zero")
                    )),
                    TextInput::new(
                        "Iterations",
                        &self.params.max_iterations.to_string(),
                        |max_iters| match max_iters.parse() {
                            Ok(mi) => {
                                Message::MaxItersUpdated(mi)
                            }
                            Err(e) => {
                                Message::PushNotification(e.to_string())
                            }
                        }
                    )
                    .on_submit(Message::ReRenderPressed),
                    Button::new("·2").on_press(Message::MaxItersUpdated(
                        self.params
                            .max_iterations
                            .get()
                            .saturating_mul(2)
                            .try_into()
                            .expect("doubling a number never gives zero")
                    )),
                ],
                // A checkbox for rendering the image in grayscale.
                Checkbox::new(self.params.grayscale, "Grayscale", |status| {
                    Message::GrayscaleToggled(status)
                }),
                // A slider for determining the number of samples per pixels when doing SSAA,
                // as well as a toggle for enabling or disabling SSAA.
                row![
                    Tooltip::new(
                        Slider::new(
                            2..=10,
                            self.ui_values.slider_ssaa_factor.get(),
                            |ssaa_factor| {
                                Message::SuperSamplingUpdated(
                                    ssaa_factor.try_into().expect("2..=10 is never zero"),
                                )
                            }
                        ),
                        format!("{} samples", self.ui_values.slider_ssaa_factor.get().pow(2)),
                        Position::FollowCursor
                    ),
                    Space::new(Length::Units(10), Length::Shrink),
                    Checkbox::new(self.ui_values.do_ssaa, "SSAA", |status| {
                        Message::SuperSamplingToggled(status)
                    })
                    .spacing(5),
                ],
                Space::new(Length::Shrink, Length::Units(40)),
                // A button for re-rendering the current view at full resolution,
                // as well as a checkbox for whether the user wants the image to be re-rendered
                // whenever they change a setting.
                if self.render_in_progress {
                    Button::new("rendering...")
                } else {
                    Button::new("re-render view").on_press(Message::ReRenderPressed)
                },
                Checkbox::new(self.ui_values.live_preview, "Live preview", |status| {
                    Message::LiveCheckboxToggled(status)
                }),
                Space::new(Length::Shrink, Length::Fill),
                // Finally a button for saving the current view.
                Button::new("Save current view").on_press(Message::SavePressed),
            ]
            .width(Length::FillPortion(1)),
        ]
        .into()
    }
}
