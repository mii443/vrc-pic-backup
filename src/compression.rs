use anyhow::Result;
use png::Decoder;
use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::Path
};

///
/// Compresses an image and writes it to a file
/// 
#[inline]
fn compress_image(img: Vec<u8>, dimension: (usize, usize), writer: BufWriter<File>, quality: f32) -> Result<()> {
    // Create a new JPEG compressor
    let mut compress = jpegli::Compress::new(jpegli::ColorSpace::JCS_RGB);
    compress.set_size(dimension.0, dimension.1);
    compress.set_quality(quality);

    // Start the compression
    let mut compress = compress.start_compress(writer)?;
    compress.write_scanlines(&img)?;
    compress.finish()?;

    Ok(())
}

///
/// Reads the pixels of a PNG file
/// 
#[inline]
fn get_pixels(path: &Path) -> Result<(Vec<u8>, (usize, usize))> {
    // Open and decode the PNG file
    let file = BufReader::new(File::open(path)?);
    let decoder = Decoder::new(file);
    let mut reader = decoder.read_info()?;
    let mut img_data = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut img_data)?;

    // Convert the image to RGB
    let bytes = match info.color_type {
        png::ColorType::Rgb => img_data,
        png::ColorType::Rgba => {
            // Remove the alpha channel
            let mut rgb_data = Vec::with_capacity((img_data.len() / 4) * 3);
            for chunk in img_data.chunks_exact(4) {
                rgb_data.extend_from_slice(&chunk[0..3]);
            }
            rgb_data
        }
        png::ColorType::Grayscale => img_data.into_iter().flat_map(|g| [g, g, g]).collect(),
        png::ColorType::GrayscaleAlpha => img_data
            .chunks_exact(2)
            .flat_map(|ga| [ga[0], ga[0], ga[0]])
            .collect(),
        _ => anyhow::bail!("Unsupported color type: {:?}", info.color_type),
    };

    Ok((bytes, (info.width as usize, info.height as usize)))
}

///
/// Compresses a PNG file and saves it as a JPEG file
/// 
#[inline]
pub fn compress_and_save(path: &Path, to_path: &Path, quality: f32) -> Result<()> {
    let (img, dimension) = get_pixels(path)?;
    
    let file = File::create(to_path)?;
    let writer = BufWriter::new(file);
    
    compress_image(img, dimension, writer, quality)?;

    Ok(())
}