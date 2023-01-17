mod embedded_resources;

use iced::{
    executor,
    widget::{image::Handle, Container, Image},
    Application, Command, Element, Length, Settings, Theme,
};

use image::DynamicImage;
use rayon::prelude::*;

use mandellib::{render, Frame, RenderParameters};

use embedded_resources::ICON;

fn main() {
    MandelViewer::run(Settings::default()).unwrap();
}

struct MandelViewer {
    image: Option<DynamicImage>,
}

#[derive(Debug, Clone, Copy)]
enum Message {}

impl Application for MandelViewer {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();
    type Theme = Theme;

    fn new(_flags: ()) -> (MandelViewer, Command<Self::Message>) {
        const INITIAL_X_RES: u32 = 1920;
        const INITIAL_Y_RES: u32 = 1080;
        const INITIAL_IMAG_DISTANCE: f64 = 8.0 / 3.0;
        const INITIAL_SSAA_FACTOR: u8 = 3;
        const INITIAL_MAX_ITERATIONS: u32 = 255;
        const INITIAL_REAL_CENTER: f64 = -0.75;
        const INITIAL_IMAG_CENTER: f64 = 0.0;

        let params = RenderParameters::new(
            INITIAL_X_RES.try_into().unwrap(),
            INITIAL_Y_RES.try_into().unwrap(),
            INITIAL_MAX_ITERATIONS.try_into().unwrap(),
            INITIAL_SSAA_FACTOR.try_into().unwrap(),
            false,
        )
        .unwrap();
        let frame = Frame::new(
            INITIAL_REAL_CENTER,
            INITIAL_IMAG_CENTER,
            f64::from(INITIAL_X_RES) / f64::from(INITIAL_Y_RES) * INITIAL_IMAG_DISTANCE,
            INITIAL_IMAG_DISTANCE,
        );

        let image = render(params, frame, false);

        (MandelViewer { image: Some(image) }, Command::none())
    }

    fn title(&self) -> String {
        String::from("Mandelviewer")
    }

    fn update(&mut self, _message: Self::Message) -> Command<Self::Message> {
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        let image_handle = match self.image {
            Some(ref img) => {
                Handle::from_pixels(img.width(), img.height(), image_to_vec_rgba8(img))
            }
            // If there is no rendered image, show the program icon.
            None => Handle::from_memory(ICON),
        };

        let image = Image::new(image_handle)
            .width(Length::Fill)
            .height(Length::Fill);

        Container::new(image)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}

fn image_to_vec_rgba8(image: &DynamicImage) -> Vec<u8> {
    match image {
        DynamicImage::ImageRgb8(img) => img
            .as_raw()
            .par_chunks_exact(3)
            .flat_map(|rgb| [rgb[0], rgb[1], rgb[2], u8::MAX])
            .collect(),
        DynamicImage::ImageLuma8(img) => img
            .as_raw()
            .par_iter()
            .flat_map(|&g| [g, g, g, u8::MAX])
            .collect(),
        DynamicImage::ImageRgba8(img) => img.as_raw().to_vec(),
        _ => unimplemented!("unsupported image type"),
    }
}
