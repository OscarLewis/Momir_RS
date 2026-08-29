use image::{DynamicImage, ImageBuffer, ImageReader};
use std::io::Write;
use std::net::TcpStream;
use tracing::debug;

const PRINTER_HOST: &str = "192.168.2.47";
// const PRINTER_HOST: &str = "0.0.0.0";
const PRINTER_PORT: u16 = 9100;

const CARD_IMAGE: &str =
    "/home/oscar/Documents/Projects/momir_rs_workspace/oracle_escpos/renders/card.png";

const BIG_CARD_IMAGE: &str =
    "/home/oscar/Documents/Projects/momir_rs_workspace/oracle_escpos/renders/beluna_card.png";

const MDFC_IMAGE: &str = "/home/oscar/Documents/Projects/momir_rs_workspace/oracle_escpos/renders/miles_morales_card.png";

pub fn print_img(
    image: ImageBuffer<image::Rgb<u8>, Vec<u8>>,
    printer_host: &str,
    printer_port: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    let dyn_img = DynamicImage::ImageRgb8(image).rotate270();
    let raw_bytes = encode_gs_v0_image(&dyn_img);
    send_raw_bytes_throttled(&raw_bytes, printer_host, printer_port)?;

    debug!(
        addr = PRINTER_HOST,
        port = PRINTER_PORT,
        "Successfully printed full card image via raw GS v 0 raster mode"
    );
    Ok(())
}

/// Helper function to load an image file from disk and orient it for vertical printing (rotate 270 degrees).
fn load_image(path: &str) -> Result<image::DynamicImage, Box<dyn std::error::Error>> {
    Ok(ImageReader::open(path)?.decode()?.rotate270())
}

/// High-level test printing a full card image using raw raster mode (GS v 0) and throttled network delivery.
pub fn test_img_print() -> Result<(), Box<dyn std::error::Error>> {
    let image = load_image(CARD_IMAGE)?;
    let raw_bytes = encode_gs_v0_image(&image);

    send_raw_bytes_throttled(&raw_bytes, PRINTER_HOST, PRINTER_PORT)?;

    debug!(
        addr = PRINTER_HOST,
        port = PRINTER_PORT,
        "Successfully printed full card image via raw GS v 0 raster mode"
    );
    Ok(())
}

/// Executes a raw network raster print test using encoded `GS v 0` bytes.
pub fn test_img_print_raw_raster() -> Result<(), Box<dyn std::error::Error>> {
    let image = load_image(BIG_CARD_IMAGE)?;
    // let raw_bytes = encode_esc_star_image(&image, 576);
    let raw_bytes = encode_gs_v0_image(&image);

    send_raw_bytes_throttled(&raw_bytes, PRINTER_HOST, PRINTER_PORT)?;

    debug!(
        addr = PRINTER_HOST,
        port = PRINTER_PORT,
        "Attempting 24-dot raw ESC * bit-image print without linefeeds via network"
    );
    Ok(())
}

/// High-level test printing an MDFC image via raw raster mode.
pub fn test_mdfc_img_print() -> Result<(), Box<dyn std::error::Error>> {
    let image = load_image(MDFC_IMAGE)?;

    let payload = encode_gs_v0_image(&image);

    send_raw_bytes_throttled(&payload, PRINTER_HOST, PRINTER_PORT)?;

    debug!(
        addr = PRINTER_HOST,
        port = PRINTER_PORT,
        "Successfully printed MDFC image via raw GS v 0 raster mode"
    );

    Ok(())
}

/// Connects directly to the network thermal printer, sends chunked raw byte streams with backpressure throttling,
/// feeds paper according to `PRE_CUT_FEED_LINES`, and executes a hardware cut.
fn send_raw_bytes_throttled(
    raw_bytes: &[u8],
    printer_host: &str,
    printer_port: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    // Configurable lines to feed paper forward before issuing cut command
    const PRE_CUT_FEED_LINES: u8 = 2;

    // Attempt network connection up to 3 times to allow hardware buffer/socket resets
    let mut stream = None;
    for attempt in 1..=3 {
        match TcpStream::connect((printer_host, printer_port)) {
            Ok(s) => {
                stream = Some(s);
                break;
            }
            Err(e) if attempt < 3 => {
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
            Err(e) => return Err(e.into()),
        }
    }
    let mut stream = stream.unwrap();

    stream.set_write_timeout(Some(std::time::Duration::from_secs(10)))?;

    // Hardware Init Command (ESC @ / 0x1B 0x40) to clear internal buffer state
    stream.write_all(&[0x1B, 0x40])?;

    // Stream raw payload in 4096-byte chunks with 25ms pauses to prevent network printer buffer overflow
    // Fall back to 512 if stuff starts breaking;
    // const CHUNK_SIZE: usize = 512;
    const CHUNK_SIZE: usize = 4096;
    for chunk in raw_bytes.chunks(CHUNK_SIZE) {
        stream.write_all(chunk)?;
        stream.flush()?;
        std::thread::sleep(std::time::Duration::from_millis(25));
    }

    // Brief delay to allow print head to finish physical movement before linefeed
    std::thread::sleep(std::time::Duration::from_millis(100));

    // ESC d <PRE_CUT_FEED_LINES> (Feed N lines) + GS V 65 0 (Explicit Full Cut command)
    let cut_command = [0x1B, 0x64, PRE_CUT_FEED_LINES, 0x1D, 0x56, 0x41, 0x00];
    stream.write_all(&cut_command)?;
    stream.flush()?;

    // Allow cutter motor hardware to complete cycle before TCP socket drops naturally
    std::thread::sleep(std::time::Duration::from_millis(250));

    Ok(())
}

/// Encodes an image using GS v 0 raster mode (0x1D 0x76 0x30 0x00).
/// Native 203 DPI raster streaming; bypasses motor stepping and line-spacing quirks.
pub fn encode_gs_v0_image(img: &DynamicImage) -> Vec<u8> {
    let luma = img.to_luma8();
    let (width, height) = luma.dimensions();

    // Round up pixel width to nearest byte boundary (8 pixels per byte)
    let width_bytes = (width + 7) / 8;
    let threshold = 128u8;

    let mut stream = Vec::new();

    // GS v 0 Command Header: GS v 0 m xL xH yL yH
    // Mode 0 (0x00) = Normal 203x203 DPI raster mode
    let x_l = (width_bytes & 0xFF) as u8;
    let x_h = ((width_bytes >> 8) & 0xFF) as u8;
    let y_l = (height & 0xFF) as u8;
    let y_h = ((height >> 8) & 0xFF) as u8;

    stream.extend_from_slice(&[0x1D, 0x76, 0x30, 0x00, x_l, x_h, y_l, y_h]);

    // Construct monochrome bit array (1 = black/dot printed, 0 = white/blank) packed MSB-first per byte
    for y in 0..height {
        for x_byte in 0..width_bytes {
            let mut byte_val = 0u8;
            for bit in 0..8 {
                let x = x_byte * 8 + bit;
                if x < width {
                    let pixel_val = luma.get_pixel(x, y)[0];
                    if pixel_val < threshold {
                        byte_val |= 1 << (7 - bit);
                    }
                }
            }
            stream.push(byte_val);
        }
    }

    stream
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires network printer hardware at 192.168.2.47"]
    fn test_img_print_executes() {
        let result = test_img_print();
        assert!(result.is_ok(), "Image print failed: {:?}", result.err());
    }

    #[test]
    #[ignore = "requires network printer hardware at 192.168.2.47"]
    fn test_mdfc_img_print_executes() {
        let result = test_mdfc_img_print();
        assert!(
            result.is_ok(),
            "MDFC image print failed: {:?}",
            result.err()
        );
    }

    #[test]
    #[ignore = "requires network printer hardware at 192.168.2.47"]
    fn test_img_print_raw_raster_executes() {
        let result = test_img_print_raw_raster();
        assert!(
            result.is_ok(),
            "24-dot raw raster image print failed: {:?}",
            result.err()
        );
    }
}
