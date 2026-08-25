use escpos::{
    driver::NetworkDriver,
    printer::Printer,
    printer_options::PrinterOptions,
    utils::{DebugMode, Protocol},
};
use image::ImageReader;
use tempfile::NamedTempFile;
use tracing::debug;

pub fn test_network_receipt_print() -> Result<(), Box<dyn std::error::Error>> {
    let host = "127.0.0.1";
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
        .writeln("Test Text")?
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
