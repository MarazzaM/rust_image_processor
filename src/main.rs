use std::env;
use std::fs::File;
use std::path::Path;
use image::{GenericImageView, RgbaImage}; // Removed imageops import
use image::imageops::overlay;

fn main() {
    let args: Vec<String> = env::args().collect();
    let (input_dir, output_dir) = match args.len() {
        3 => (args[1].clone(), args[2].clone()), // Use command-line arguments if provided
        _ => {
            // Prompt the user for input and output directories
            let default_input = "C:\\input";
            let default_output = "C:\\output";
            println!("Enter input directory (default: {}):", default_input);
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            let input_dir = if input.trim().is_empty() { default_input.to_string() } else { input.trim().to_string() };

            println!("Enter output directory (default: {}):", default_output);
            let mut output = String::new();
            std::io::stdin().read_line(&mut output).unwrap();
            let output_dir = if output.trim().is_empty() { default_output.to_string() } else { output.trim().to_string() };

            (input_dir, output_dir)
        }
    };

    // Processing each image in the input directory
    if let Ok(entries) = std::fs::read_dir(&input_dir) {
        entries.for_each(|entry| {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file() {
                    process_image(&path, &output_dir);
                }
            }
        });
    }
}

fn process_image(input_path: &Path, output_dir: &str) {
    let mut img = match image::open(input_path) {
        Ok(img) => img,
        Err(_) => {
            eprintln!("Failed to open {:?}", input_path);
            return;
        },
    };

    let watermark = match image::open("./assets/logo.png") { // Adjust path as necessary
        Ok(watermark) => watermark,
        Err(_) => {
            eprintln!("Failed to load watermark image");
            return;
        },
    };

    let (ix, iy) = img.dimensions();
    let (wx, wy) = watermark.dimensions();

    // Determine the target dimensions based on whether the image is horizontal or vertical
    let (target_width, target_height) = if ix > iy {
        (1600, 1200) // Horizontal image dimensions
    } else {
        (1200, 1600) // Vertical image dimensions
    };

    // Resize the image to the target dimensions
    img = img.resize(target_width, target_height, image::imageops::FilterType::Nearest);

    // Calculate the position to center the watermark
    let (ix, iy) = img.dimensions();
    let x = (ix as i64 - wx as i64) / 2;
    let y = (iy as i64 - wy as i64) / 2;

    // Resize the watermark to fit the image and make it semi-transparent
    let watermark_resized = watermark.resize(wx, wy, image::imageops::FilterType::Nearest);
    let mut watermark_resized = watermark_resized.into_rgba8();
    watermark_resized.rchunks_mut(4).for_each(|pixel| {
        pixel[3] = (pixel[3] as f32 * 0.8) as u8; // Adjust transparency here
    });

    // Use overlay to apply the watermark
    overlay(&mut img, &RgbaImage::from_raw(wx, wy, watermark_resized.into_vec()).unwrap(), x as i64, y as i64);

    let output_path = Path::new(output_dir).join(input_path.file_name().unwrap());
    let ref mut fout = File::create(&output_path).unwrap();

    // Compressing the image as JPEG with quality 85
    match img.write_to(fout, image::ImageOutputFormat::Jpeg(85)) {
        Ok(_) => (),
        Err(_) => eprintln!("Failed to write output image."),
    };
}
