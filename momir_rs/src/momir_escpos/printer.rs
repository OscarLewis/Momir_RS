use escpos::{
    driver::NetworkDriver,
    printer::Printer,
    printer_options::PrinterOptions,
    utils::{DebugMode, Protocol},
};
use image::ImageReader;
use tempfile::NamedTempFile;

pub fn test_receipt_print() -> Result<(), Box<dyn std::error::Error>> {
    let driver = NetworkDriver::open("127.0.0.1", 9100, None)?;

    let image = ImageReader::open(
        "/home/oscar/Documents/Projects/momir_rs_workspace/momir_rs/static/images/momir_small.png",
    )?
    .decode()?
    .rotate270();

    let temp_file = NamedTempFile::with_suffix(".png")?;
    image.save(temp_file.path())?;

    let image_path = temp_file
        .path()
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Invalid image path"))?;

    Printer::new(driver, Protocol::default(), Some(PrinterOptions::default()))
        .debug_mode(Some(DebugMode::Hex))
        .init()?
        .writeln("Test Text")?
        .feed()?
        .bit_image(image_path)?
        .feed()?
        .print_cut()?;
    Ok(())
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_print() -> Result<(), Box<dyn std::error::Error>> {
//         test_receipt_print()?;
//         Ok(())
//     }
// }
