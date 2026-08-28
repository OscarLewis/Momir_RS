use std::path::PathBuf;

use escpos::{
    driver::NetworkDriver,
    printer::Printer,
    printer_options::PrinterOptions,
    utils::{BitImageOption, BitImageSize, DebugMode, JustifyMode, Protocol},
};
use image::ImageReader;
use tempfile::NamedTempFile;
use tracing::debug;

pub fn test_network_receipt_print() -> Result<(), Box<dyn std::error::Error>> {
    let host = "192.168.2.47";
    let host_port = 9100;
    let driver = NetworkDriver::open(host, host_port, None)?;
    let mut printer = Printer::new(driver, Protocol::default(), Some(PrinterOptions::default()));

    let image = ImageReader::open(
        "/home/oscar/Documents/Projects/momir_rs_workspace/momir_rs/static/images/momir_small.png",
    )?
    .decode()?
    .rotate270();

    let temp_file = NamedTempFile::with_suffix(".png")?;
    image.to_luma8().save(temp_file.path())?;

    let image_path = temp_file.path().to_str().ok_or("Invalid image path")?;

    // TODO
    // Consider using 'swash' to build text then send to printer as
    // a rasterized Image instead of trying to get everything composed
    // as print commands.
    // https://docs.rs/swash/latest/swash/

    printer
        .debug_mode(Some(DebugMode::Hex))
        .init()?
        .flip(true)?
        .write("Test Text")?
        .flip(false)?
        .write("Test two")?
        .feed()?
        .bit_image(image_path)?
        .feed()?
        .print_cut()?;

    debug!(
        addr = host,
        port = host_port,
        "Attempting to print test page via network"
    );

    Ok(())
}

pub fn test_img_print() -> Result<(), Box<dyn std::error::Error>> {
    let host = "192.168.2.47";
    let host_port = 9100;
    let driver = NetworkDriver::open(host, host_port, None)?;
    let mut printer = Printer::new(driver, Protocol::default(), Some(PrinterOptions::default()));

    let image = ImageReader::open(
        "/home/oscar/Documents/Projects/momir_rs_workspace/oracle_escpos/renders/card.png",
    )?
    .decode()?
    .rotate270();

    let temp_file = NamedTempFile::with_suffix(".png")?;
    // image.to_luma8().save(temp_file.path())?;
    image.save(temp_file.path())?;

    let image_path = temp_file.path().to_str().ok_or("Invalid image path")?;

    // TODO
    // Consider using 'swash' to build text then send to printer as
    // a rasterized Image instead of trying to get everything composed
    // as print commands.
    // https://docs.rs/swash/latest/swash/

    printer
        .debug_mode(Some(DebugMode::Hex))
        .init()?
        .justify(JustifyMode::LEFT)?
        .bit_image_option(
            image_path,
            BitImageOption::new(Some(576), None, BitImageSize::Normal)?,
        )?
        .feed()?
        .print_cut()?;

    debug!(
        addr = host,
        port = host_port,
        "Attempting to print test page via network"
    );

    Ok(())
}
pub fn test_mdfc_img_print() -> Result<(), Box<dyn std::error::Error>> {
    // let host = "0.0.0.0";

    let host = "192.168.2.47";
    let host_port = 9100;
    let driver = NetworkDriver::open(host, host_port, None)?;
    let mut printer = Printer::new(driver, Protocol::default(), Some(PrinterOptions::default()));

    let image = ImageReader::open(
    "/home/oscar/Documents/Projects/momir_rs_workspace/oracle_escpos/renders/miles_morales_card.png",
)?
.decode()?
.rotate270();

    let width = image.width();
    let height = image.height();
    let half_width = width / 2;

    let half_height = height / 2;
    let front = image.crop_imm(0, 0, width, half_height);
    let back = image.crop_imm(0, half_height, width, half_height);

    let temp_left = NamedTempFile::with_suffix(".png")?;
    let temp_right = NamedTempFile::with_suffix(".png")?;

    let right_debug = PathBuf::from("/home/oscar/Downloads/right.png");
    let left_debug = PathBuf::from("/home/oscar/Downloads/left.png");
    front.save(left_debug)?;
    front.save(temp_left.path())?;
    back.save(temp_right.path())?;
    back.save(right_debug)?;

    let left_path = temp_left.path().to_str().ok_or("Invalid left path")?;
    let right_path = temp_right.path().to_str().ok_or("Invalid right path")?;

    printer
        .debug_mode(Some(DebugMode::Hex))
        .init()?
        .justify(JustifyMode::LEFT)?
        .bit_image_option(
            left_path,
            BitImageOption::new(Some(576), None, BitImageSize::Normal)?,
        )?
        .print()?
        .feed()?
        .bit_image_option(
            right_path,
            BitImageOption::new(Some(576), None, BitImageSize::Normal)?,
        )?
        .feed()?
        .print_cut()?;

    debug!(
        addr = host,
        port = host_port,
        "Attempting to print MDFC test page via network"
    );

    Ok(())
}
