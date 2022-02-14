use image::{
    imageops::filter3x3, io::Reader as ImageReader, DynamicImage, GenericImage, ImageBuffer, Pixel,
    RgbaImage,
};
use imageproc::{
    definitions::Image, drawing::draw_text_mut, filter::gaussian_blur_f32, map::map_colors2,
};
use rusttype::{Font, Scale};

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    command: Commands,
    a_blur: Option<f32>,
    b_blur: Option<f32>,
    c_blur: Option<f32>,
}

#[derive(Subcommand)]
enum Commands {
    File {
        file_a: String,
        file_b: String,
    },
    Text {
        msg1: String,
        msg2: String,
        msg3: Option<String>,
    },
}

const IDENTITY_MINUS_LAPLACIAN: [f32; 9] = [0.0, -1.0, 0.0, -1.0, 5.0, -1.0, 0.0, -1.0, 0.0];
const TEXT_COLOR_R: image::Rgba<u8> = image::Rgba([255, 0, 0, 255]);
const TEXT_COLOR_B: image::Rgba<u8> = image::Rgba([0, 0, 255, 255]);
const TEXT_COLOR_G: image::Rgba<u8> = image::Rgba([0, 255, 0, 255]);
const TEXT_COLOR_W: image::Rgba<u8> = image::Rgba([255, 255, 255, 255]);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let (img1, img2, img3) = match &args.command {
        Commands::File { file_a, file_b } => {
            let img1 = ImageReader::open(file_a)?.decode()?;
            let img2 = ImageReader::open(file_b)?.decode()?;
            (img1, img2, None)
        }
        Commands::Text { msg1, msg2, msg3 } => {
            let msg3_len = if let Some(msg) = msg3 { msg.len() } else { 0 };
            let len = msg1.len().max(msg2.len()).max(msg3_len);
            let width = ((400 * len) / 4) as u32;

            let img1 = draw_message(
                msg1.to_string(),
                width,
                200,
                20,
                35,
                Scale::uniform(150.0),
                TEXT_COLOR_R,
            );
            let img2 = draw_message(
                msg2.to_string(),
                width,
                200,
                20,
                35,
                Scale::uniform(150.0),
                TEXT_COLOR_G,
            );
            let img3 = if let Some(msg3) = msg3 {
                Some(draw_message(
                    msg3.to_string(),
                    width,
                    200,
                    20,
                    35,
                    Scale::uniform(150.0),
                    TEXT_COLOR_B,
                ))
            } else {
                None
            };
            (img1, img2, img3)
        }
    };
    let img1 = low_pass(img1, args.a_blur.unwrap_or(4.5));
    let img2 = high_pass(img2, args.b_blur.unwrap_or(0.545));
    img1.save("a.jpg")?;
    img2.save("b.jpg")?;
    let t = if let Some(img3) = img3 {
        let img3 = high_pass(img3, args.c_blur.unwrap_or(0.0));
        img3.save("c.jpg")?;
        overlay3(img1, img2, img3)
    } else {
        overlay(img1, img2)
    };
    t.save("t.jpg")?;
    Ok(())
}
fn draw_message(
    msg: String,
    width: u32,
    height: u32,
    x: u32,
    y: u32,
    scale: Scale,
    color: image::Rgba<u8>,
) -> DynamicImage {
    let font_data: &[u8] = include_bytes!("/usr/share/fonts/FuturaLT-Bold.ttf");
    let font: Font<'static> = Font::try_from_bytes(font_data).unwrap();
    let canvas: RgbaImage = ImageBuffer::new(width, height);
    let mut img = DynamicImage::ImageRgba8(canvas);
    draw_text_mut(&mut img, color, x, y, scale, &font, &msg);
    img
}

fn clamp_sub(a: u8, b: u8, max: u8) -> u8 {
    if a < b {
        max.min(b)
    } else {
        max.min(a - b)
    }
}

fn clamp_add(a: u8, b: u8, max: u8) -> u8 {
    if (a as u16 + b as u16) > max.into() {
        max
    } else {
        a + b
    }
}

fn low_pass(img: DynamicImage, amt: f32) -> DynamicImage {
    DynamicImage::ImageRgba8(gaussian_blur_f32(&img.to_rgba8(), amt))
}

fn laplacian(amt:f32) -> [f32;9]{
    let mut v = IDENTITY_MINUS_LAPLACIAN;
    v[4] *= amt;
    v
}

fn high_pass(img: DynamicImage, amt: f32) -> DynamicImage {
    let img_impulse = filter3x3(&img, &laplacian(amt));
    let img_low = low_pass(img, amt/2.0);
    let diff = map_colors2(&img_impulse, &img_low, |mut p, q| {
        p.apply2(&q, |c1, c2| clamp_sub(c1, c2, u8::MAX));
        p.0[3] = 255;
        p
    });
    DynamicImage::ImageRgba8(diff)
}

fn overlay(a: DynamicImage, b: DynamicImage) -> DynamicImage {
    let diff = map_colors2(&a, &b, |mut p, q| {
        p.apply2(&q, |c1, c2| (clamp_add(c1, c2, u8::MAX)));
        p.0[3] = 255;
        p
    });
    DynamicImage::ImageRgba8(diff)
}

fn overlay3(a: DynamicImage, b: DynamicImage, c: DynamicImage) -> DynamicImage {
    let diff = map_colors3(&a, &b, &c, |mut p, q, r| {
        assert_eq!(p.channels().len(), q.channels().len());
        assert_eq!(p.channels().len(), r.channels().len());
        for i in 0..p.channels().len() - 1 {
            p.channels_mut()[i] = clamp_add(
                clamp_add(p.channels()[i], q.channels()[i], u8::MAX),
                r.channels()[i],
                u8::MAX,
            );
        }
        p
    });
    DynamicImage::ImageRgba8(diff)
}

fn map_colors3<I, J, K, P, Q, R, S, F>(image1: &I, image2: &J, image3: &K, f: F) -> Image<S>
where
    I: GenericImage<Pixel = P>,
    J: GenericImage<Pixel = Q>,
    K: GenericImage<Pixel = R>,
    P: Pixel,
    Q: Pixel,
    R: Pixel,
    S: Pixel + 'static,
    F: Fn(P, Q, R) -> S,
{
    assert_eq!(image1.dimensions(), image2.dimensions());

    let (width, height) = image1.dimensions();
    let mut out: ImageBuffer<S, Vec<S::Subpixel>> = ImageBuffer::new(width, height);

    for y in 0..height {
        for x in 0..width {
            unsafe {
                let p = image1.unsafe_get_pixel(x, y);
                let q = image2.unsafe_get_pixel(x, y);
                let r = image3.unsafe_get_pixel(x, y);
                out.unsafe_put_pixel(x, y, f(p, q, r));
            }
        }
    }

    out
}
