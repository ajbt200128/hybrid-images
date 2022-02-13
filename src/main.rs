use image::{imageops::filter3x3, io::Reader as ImageReader, DynamicImage, Pixel, ImageBuffer, RgbaImage};
use imageproc::{filter::gaussian_blur_f32, map::map_colors2, drawing::draw_text_mut};
use rusttype::{Font, Scale};

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    command: Commands,
    a_blur: Option<f32>,
    b_blur: Option<f32>,
}

#[derive(Subcommand)]
enum Commands {
    File { file_a: String, file_b: String },
    Text { msg1: String, msg2: String },
}

const IDENTITY_MINUS_LAPLACIAN: [f32; 9] = [0.0, -1.0, 0.0, -1.0, 5.0, -1.0, 0.0, -1.0, 0.0];
const TEXT_COLOR_R: image::Rgba<u8> = image::Rgba([255,0,0,255]);
const TEXT_COLOR_B: image::Rgba<u8> = image::Rgba([125,0,255,255]);
const TEXT_COLOR_G: image::Rgba<u8> = image::Rgba([0,255,0,255]);
const TEXT_COLOR_W: image::Rgba<u8> = image::Rgba([255,255,255,255]);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let (img1, img2) = match &args.command {
        Commands::File { file_a, file_b } => {
            let img1 = ImageReader::open(file_a)?.decode()?;
            let img2 = ImageReader::open(file_b)?.decode()?;
            (img1, img2)
        }
        Commands::Text {msg1, msg2 } =>{
            let img1 = draw_message(msg1.to_string(), 500, 200, 20, 35, Scale::uniform(150.0),TEXT_COLOR_B);
            img1.save("a.jpg")?;
            let img2 = draw_message(msg2.to_string(), 500, 200, 20, 35, Scale::uniform(150.0),TEXT_COLOR_R);
            (img1,img2)
        }
    };
    let img1 = high_pass(img1, args.a_blur.unwrap_or(0.545));
    //img1.save("a.jpg")?;
    let img2 = low_pass(img2, args.b_blur.unwrap_or(4.5));
    img2.save("b.jpg")?;
    let t = overlay(img1, img2);
    t.save("t.jpg")?;
    Ok(())
}
fn draw_message(msg:String,width:u32,height:u32,x:u32,y:u32,scale:Scale,color:image::Rgba<u8>) -> DynamicImage{
    let font_data: &[u8] = include_bytes!("/usr/share/fonts/FuturaLT-Bold.ttf");
    let font: Font<'static> = Font::try_from_bytes(font_data).unwrap();
    let canvas:RgbaImage= ImageBuffer::new(width,height);
    let mut img = DynamicImage::ImageRgba8(canvas.clone());
    let mut img_outline = DynamicImage::ImageRgba8(canvas);
    draw_text_mut(&mut img, color, x, y, scale, &font, &msg);
    draw_text_mut(&mut img_outline, TEXT_COLOR_W, x, y, scale, &font, &msg);
    let clone = img.clone();
    let img = low_pass(img_outline, 10.0);
    overlay(img,clone)
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

fn high_pass(img: DynamicImage, amt: f32) -> DynamicImage {
    let img_impulse = filter3x3(&img, &IDENTITY_MINUS_LAPLACIAN);
    let img_low = low_pass(img, amt);
    let diff = map_colors2(&img_impulse, &img_low, |mut p, q| {
        p.apply2(&q, |c1, c2| clamp_sub(c1, c2, u8::MAX));
        p.0[3] = 255;
        p
    });
    DynamicImage::ImageRgba8(diff)
}

fn overlay(a: DynamicImage, b: DynamicImage) -> DynamicImage {
    let diff = map_colors2(&a, &b, |mut p, q| {
        p.apply2(&q, |c1, c2| (clamp_add(c1, c2, u8::MAX)) / 2);
        p
    });
    DynamicImage::ImageRgba8(diff)
}
