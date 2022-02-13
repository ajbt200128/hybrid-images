use image::{io::Reader as ImageReader, DynamicImage, imageops::{filter3x3, self},ImageBuffer, Luma, Pixel};
use image::Rgba;
use imageproc::{filter::{gaussian_blur_f32, sharpen_gaussian}, map::map_colors2};

const IDENTITY_MINUS_LAPLACIAN: [f32; 9] = [0.0, -1.0, 0.0, -1.0, 5.0, -1.0, 0.0, -1.0, 0.0];
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut img1 = ImageReader::open("images/hybridImages/bear.jpg")?.decode()?;
    let img2 = ImageReader::open("images/hybridImages/wrighton.jpg")?.decode()?;
    // Create a window with default options and display the image.
    let img1 = high_pass(img1);
    img1.save("a.jpg")?;
    let img2 = low_pass(img2,13.0);
    img2.save("b.jpg")?;
    let t = overlay(img1, img2);
    t.save("t.jpg")?;
    Ok(())
}

fn clamp_sub(a:u8,b:u8,max:u8)->u8{
    if a < b {
        max.min(b)
    }else{
        max.min(a-b)
    }
}

fn clamp_add(a:u8,b:u8,max:u8)->u8{
    if (a as u16 + b as u16) > max.into(){
        max
    }else{
        a+b
    }
}

fn low_pass(img:DynamicImage,amt:f32) -> DynamicImage{
    DynamicImage::ImageRgba8(gaussian_blur_f32(&img.to_rgba8(), amt))
}

fn high_pass(img:DynamicImage) -> DynamicImage{
    let img_impulse = filter3x3(&img, &IDENTITY_MINUS_LAPLACIAN);
    let img_low = low_pass(img,0.6);
    let diff = map_colors2(&img_impulse,&img_low, |mut p,q|
                           {
                               p.apply2(&q, |c1,c2| {
                                   clamp_sub(c1, c2, u8::MAX)
                               });
                               p.0[3] = 255;
                               p
                           });
    DynamicImage::ImageRgba8(diff)
}

fn overlay(a:DynamicImage,b:DynamicImage) -> DynamicImage{
    let diff = map_colors2(&a, &b, |mut p,q| {
        p.apply2(&q, |c1,c2| {
            (clamp_add(c1, c2, u8::MAX))/2
        });
        p
    });
    DynamicImage::ImageRgba8(diff)
}
