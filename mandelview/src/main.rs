use iced::{
    executor,
    widget::{image::Handle, Container, Image},
    Application, Command, Element, Length, Settings, Theme,
};

use rayon::prelude::*;

use mandellib::{render, Frame, RenderParameters};

fn main() {
    MandelViewer::run(Settings::default()).unwrap();
}

struct MandelViewer;

impl Application for MandelViewer {
    type Executor = executor::Default;
    type Message = ();
    type Flags = ();
    type Theme = Theme;

    fn new(_flags: ()) -> (MandelViewer, Command<Self::Message>) {
        (MandelViewer, Command::none())
    }

    fn title(&self) -> String {
        String::from("Mandelviewer")
    }

    fn update(&mut self, _message: Self::Message) -> Command<Self::Message> {
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        const X_RES: u32 = 1920;
        const Y_RES: u32 = 1080;

        let params = RenderParameters::new(
            X_RES.try_into().unwrap(),
            Y_RES.try_into().unwrap(),
            255.try_into().unwrap(),
            3.try_into().unwrap(),
            false,
        )
        .unwrap();
        let l = 8.0 / 3.0;
        let frame = Frame::new(-0.75, 0.0, f64::from(X_RES) / f64::from(Y_RES) * l, l);
        let image_handle = Handle::from_pixels(
            X_RES,
            Y_RES,
            rgb8_to_rgba8(render(params, frame, false).into_bytes()),
        );

        // let image_handle = match f(1920, 1080) {
        //     DynamicImage::ImageRgb8(img) => {
        //         Handle::from_pixels(1920, 1080, rgb8_to_rgba8(img.into_raw()))
        //     }
        //     DynamicImage::ImageLuma8(img) => {
        //         Handle::from_pixels(1920, 1080, rgb8_to_rgba8(img.into_raw()))
        //     }
        //     _ => Handle::from_memory(PRERENDERED),
        // };

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

fn rgb8_to_rgba8(pixel_bytes: Vec<u8>) -> Vec<u8> {
    pixel_bytes
        .par_chunks_exact(3)
        .flat_map(|rgb| [rgb[0], rgb[1], rgb[2], u8::MAX])
        .collect()
}
