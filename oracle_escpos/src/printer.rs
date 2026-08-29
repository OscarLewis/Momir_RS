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

const PRINTER_HOST: &str = "192.168.2.47";
const PRINTER_PORT: u16 = 9100;

const MOMIR_IMAGE: &str =
    "/home/oscar/Documents/Projects/momir_rs_workspace/momir_rs/static/images/momir_small.png";

const CARD_IMAGE: &str =
    "/home/oscar/Documents/Projects/momir_rs_workspace/oracle_escpos/renders/card.png";

const MDFC_IMAGE: &str = "/home/oscar/Documents/Projects/momir_rs_workspace/oracle_escpos/renders/miles_morales_card.png";

fn printer() -> Result<Printer<NetworkDriver>, Box<dyn std::error::Error>> {
    let driver = NetworkDriver::open(PRINTER_HOST, PRINTER_PORT, None)?;

    Ok(Printer::new(
        driver,
        Protocol::default(),
        Some(PrinterOptions::default()),
    ))
}

fn load_image(path: &str) -> Result<image::DynamicImage, Box<dyn std::error::Error>> {
    Ok(ImageReader::open(path)?.decode()?.rotate270())
}

fn save_temp_image(
    image: &image::DynamicImage,
) -> Result<NamedTempFile, Box<dyn std::error::Error>> {
    let temp_file = NamedTempFile::with_suffix(".png")?;
    image.save(temp_file.path())?;
    Ok(temp_file)
}

fn image_path(file: &NamedTempFile) -> Result<&str, Box<dyn std::error::Error>> {
    file.path()
        .to_str()
        .ok_or_else(|| "Invalid image path".into())
}

fn log_print_attempt(message: &str) {
    debug!(addr = PRINTER_HOST, port = PRINTER_PORT, "{message}");
}

pub fn test_network_receipt_print() -> Result<(), Box<dyn std::error::Error>> {
    let mut printer = printer()?;
    let image = load_image(MOMIR_IMAGE)?;

    let temp_file = {
        let image = image.to_luma8();
        let temp_file = NamedTempFile::with_suffix(".png")?;
        image.save(temp_file.path())?;
        temp_file
    };

    let image_path = image_path(&temp_file)?;

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

    log_print_attempt("Attempting to print test page via network");

    Ok(())
}

pub fn test_img_print() -> Result<(), Box<dyn std::error::Error>> {
    let mut printer = printer()?;
    let image = load_image(CARD_IMAGE)?;
    let temp_file = save_temp_image(&image)?;
    let image_path = image_path(&temp_file)?;

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

    log_print_attempt("Attempting to print test page via network");

    Ok(())
}

pub fn test_mdfc_img_print() -> Result<(), Box<dyn std::error::Error>> {
    let mut printer = printer()?;
    let image = load_image(MDFC_IMAGE)?;

    let width = image.width();
    let half_height = image.height() / 2;

    let front = image.crop_imm(0, 0, width, half_height);
    let back = image.crop_imm(0, half_height, width, half_height);

    let left_temp = save_temp_image(&front)?;
    let right_temp = save_temp_image(&back)?;

    let left_path = image_path(&left_temp)?;
    let right_path = image_path(&right_temp)?;

    let left_debug = PathBuf::from("/home/oscar/Downloads/left.png");
    let right_debug = PathBuf::from("/home/oscar/Downloads/right.png");

    front.save(left_debug)?;
    back.save(right_debug)?;

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

    log_print_attempt("Attempting to print MDFC test page via network");

    Ok(())
}
