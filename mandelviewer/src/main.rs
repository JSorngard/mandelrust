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
use image::{ColorType, DynamicImage, ImageFormat};
use nonzero_ext::nonzero;
use rfd::FileDialog;

// Initial view settings
const INITIAL_SSAA_FACTOR: NonZeroU8 = nonzero!(3_u8);
const INITIAL_MAX_ITERATIONS: NonZeroU32 = nonzero!(256_u32);
const INITIAL_X_RES: NonZeroU32 = nonzero!(1920_u32);
const INITIAL_Y_RES: NonZeroU32 = nonzero!(1080_u32);
const INITIAL_IMAG_DISTANCE: f64 = 8.0 / 3.0;
const INITIAL_REAL_CENTER: f64 = -0.75;
const INITIAL_IMAG_CENTER: f64 = 0.0;

// Program settings
const PROGRAM_NAME: &str = "Mandelviewer";
const PREVIEW_RES: NonZeroU32 = nonzero!(480_u32);
const NOTIFICATION_DURATION: Duration = Duration::from_secs(5);

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
    aspect_ratio: f64,
    view_region: Frame,
    render_in_progress: bool,
    notifications: Vec<String>,
    ui_values: UIValues,
}

#[derive(Debug, Clone)]
enum NotificationAction {
    Push(String),
    Pop,
}

#[derive(Debug, Clone)]
enum SSAAAction {
    Toggled(bool),
    NumSamplesUpdated(NonZeroU8),
}

#[derive(Debug, Clone)]
enum RenderAction {
    Started,
    Finished(DynamicImage),
}

#[derive(Debug, Clone)]
enum Message {
    Render(RenderAction),
    MaxItersUpdated(NonZeroU32),
    Notification(NotificationAction),
    LiveCheckboxToggled(bool),
    GrayscaleToggled(bool),
    SavePressed,
    VerticalResolutionUpdated(NonZeroU32),
    SuperSampling(SSAAAction),
}

async fn background_timer(duration: Duration) {
    std::thread::sleep(duration)
}

impl MandelViewer {
    fn with_new_resolution(&self, y_res: NonZeroU32) -> Result<RenderParameters, TryFromIntError> {
        let mut new_params = self.params;
        new_params.y_resolution = y_res.try_into()?;
        new_params.x_resolution =
            ((f64::from(y_res.get()) * self.aspect_ratio) as u32).try_into()?;
        Ok(new_params)
    }

    fn push_notification(&mut self, text: String) -> Command<<Self as Application>::Message> {
        self.notifications.push(text);
        Command::perform(background_timer(NOTIFICATION_DURATION), |_| {
            Message::Notification(NotificationAction::Pop)
        })
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
            ColorType::Rgb8,
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
                aspect_ratio: f64::from(INITIAL_X_RES.get()) / f64::from(INITIAL_Y_RES.get()),
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
                Command::perform(render(params, view_region, false), |img| {
                    Message::Render(RenderAction::Finished(img))
                }),
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
                            self.with_new_resolution(PREVIEW_RES)
                                .expect("PREVIEW_RES is a valid resolution"),
                            self.view_region,
                            false,
                        ),
                        |img| Message::Render(RenderAction::Finished(img)),
                    )
                } else {
                    Command::none()
                }
            }
            Message::Render(action) => match action {
                RenderAction::Started => {
                    self.render_in_progress = true;
                    // Clear viewer to save memory
                    self.image = None;
                    Command::perform(render(self.params, self.view_region, false), |img| {
                        Message::Render(RenderAction::Finished(img))
                    })
                }
                RenderAction::Finished(img) => {
                    self.render_in_progress = false;
                    self.image = Some(img);
                    Command::none()
                }
            },
            Message::Notification(action) => match action {
                NotificationAction::Push(e) => self.push_notification(e),
                NotificationAction::Pop => {
                    self.notifications.drain(..=0);
                    Command::none()
                }
            },
            Message::LiveCheckboxToggled(state) => {
                self.ui_values.live_preview = state;
                Command::none()
            }
            Message::GrayscaleToggled(state) => {
                self.params.color_type = if state {
                    ColorType::L8
                } else {
                    ColorType::Rgb8
                };
                if self.ui_values.live_preview {
                    Command::perform(
                        render(
                            self.with_new_resolution(PREVIEW_RES)
                                .expect("PREVIEW_RES is a valid resolution"),
                            self.view_region,
                            false,
                        ),
                        |img| Message::Render(RenderAction::Finished(img)),
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
            Message::VerticalResolutionUpdated(y_res) => match self.with_new_resolution(y_res) {
                Ok(params) => {
                    if params.x_resolution.u32.get() * params.y_resolution.u32.get() * 4
                        <= 1000000000
                    {
                        self.params = params;
                        if self.ui_values.live_preview {
                            Command::perform(render(self.params, self.view_region, false), |img| {
                                Message::Render(RenderAction::Finished(img))
                            })
                        } else {
                            Command::none()
                        }
                    } else {
                        self.push_notification("the resolution is too large".into())
                    }
                }
                Err(e) => self.push_notification(e.to_string()),
            },
            Message::SuperSampling(action) => match action {
                SSAAAction::NumSamplesUpdated(ssaa_factor) => {
                    self.ui_values.slider_ssaa_factor = ssaa_factor;
                    if self.ui_values.live_preview && self.ui_values.do_ssaa {
                        self.params.sqrt_samples_per_pixel = self.ui_values.slider_ssaa_factor;
                        Command::perform(
                            render(
                                self.with_new_resolution(PREVIEW_RES)
                                    .expect("PREVIEW_RES is a valid resolution"),
                                self.view_region,
                                false,
                            ),
                            |img| Message::Render(RenderAction::Finished(img)),
                        )
                    } else {
                        Command::none()
                    }
                }
                SSAAAction::Toggled(do_ssaa) => {
                    self.ui_values.do_ssaa = do_ssaa;
                    if !self.ui_values.do_ssaa {
                        self.params.sqrt_samples_per_pixel = 1.try_into().expect("1 is not zero");
                    } else {
                        self.params.sqrt_samples_per_pixel = self.ui_values.slider_ssaa_factor;
                    };

                    if self.ui_values.live_preview {
                        Command::perform(
                            render(
                                self.with_new_resolution(PREVIEW_RES)
                                    .expect("PREVIEW_RES is a valid resolution"),
                                self.view_region,
                                false,
                            ),
                            |img| Message::Render(RenderAction::Finished(img)),
                        )
                    } else {
                        Command::none()
                    }
                }
            },
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
            Space::new(Length::Fixed(20.0), Length::Shrink),
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
                            Err(e) =>
                                Message::Notification(NotificationAction::Push(e.to_string())),
                        }
                    )
                    .on_submit(Message::Render(RenderAction::Started)),
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
                                Message::Notification(NotificationAction::Push(e.to_string()))
                            }
                        }
                    )
                    .on_submit(Message::Render(RenderAction::Started)),
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
                Checkbox::new("Grayscale", !self.params.color_type.has_color(), |status| {
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
                                Message::SuperSampling(SSAAAction::NumSamplesUpdated(
                                    ssaa_factor.try_into().expect("2..=10 is never zero"),
                                ))
                            }
                        ),
                        format!("{} samples", self.ui_values.slider_ssaa_factor.get().pow(2)),
                        Position::FollowCursor
                    ),
                    Space::new(Length::Fixed(10.0), Length::Shrink),
                    Checkbox::new("SSAA", self.ui_values.do_ssaa, |status| {
                        Message::SuperSampling(SSAAAction::Toggled(status))
                    })
                    .spacing(5),
                ],
                Space::new(Length::Shrink, Length::Fixed(40.0)),
                // A button for re-rendering the current view at full resolution,
                // as well as a checkbox for whether the user wants the image to be re-rendered
                // whenever they change a setting.
                if self.render_in_progress {
                    Button::new("rendering...")
                } else {
                    Button::new("re-render view").on_press(Message::Render(RenderAction::Started))
                },
                Checkbox::new("Live preview", self.ui_values.live_preview, |status| {
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
