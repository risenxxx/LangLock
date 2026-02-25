use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() {
    // Generate icon.ico file
    generate_icon();

    // Embed the Windows resources (manifest + icon)
    embed_resource::compile("langlock.rc", embed_resource::NONE);
}

/// Generates a simple 32x32 ICO file with an "L" on blue background.
fn generate_icon() {
    let out_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let icon_path = Path::new(&out_dir).join("icon.ico");

    // Skip if icon already exists
    if icon_path.exists() {
        return;
    }

    const SIZE: u32 = 32;
    let mut rgba = vec![0u8; (SIZE * SIZE * 4) as usize];

    for y in 0..SIZE {
        for x in 0..SIZE {
            let idx = ((y * SIZE + x) * 4) as usize;

            // Background color (blue) - BGRA format for ICO
            let (b, g, r) = (235, 99, 37); // #2563eb in BGR

            // Draw "L" shape in white
            let is_l = (x >= 8 && x <= 12 && y >= 6 && y <= 24)
                || (x >= 8 && x <= 22 && y >= 20 && y <= 24);

            if is_l {
                // White (BGRA)
                rgba[idx] = 255;     // B
                rgba[idx + 1] = 255; // G
                rgba[idx + 2] = 255; // R
                rgba[idx + 3] = 255; // A
            } else {
                // Blue background (BGRA)
                rgba[idx] = b;
                rgba[idx + 1] = g;
                rgba[idx + 2] = r;
                rgba[idx + 3] = 255;
            }
        }
    }

    // Create ICO file
    let ico_data = create_ico(&rgba, SIZE);
    let mut file = File::create(&icon_path).expect("Failed to create icon.ico");
    file.write_all(&ico_data).expect("Failed to write icon.ico");
}

/// Creates a minimal ICO file from BGRA pixel data.
fn create_ico(bgra: &[u8], size: u32) -> Vec<u8> {
    let mut ico = Vec::new();

    // ICO Header (6 bytes)
    ico.extend_from_slice(&[0, 0]); // Reserved
    ico.extend_from_slice(&[1, 0]); // Type: 1 = ICO
    ico.extend_from_slice(&[1, 0]); // Number of images: 1

    // Image directory entry (16 bytes)
    ico.push(size as u8);  // Width (0 = 256)
    ico.push(size as u8);  // Height (0 = 256)
    ico.push(0);           // Color palette: 0 = no palette
    ico.push(0);           // Reserved
    ico.extend_from_slice(&[1, 0]); // Color planes: 1
    ico.extend_from_slice(&[32, 0]); // Bits per pixel: 32

    // Calculate BMP data size
    let row_size = size * 4; // BGRA = 4 bytes per pixel
    let pixel_data_size = row_size * size;
    let bmp_header_size: u32 = 40; // BITMAPINFOHEADER
    let image_size = bmp_header_size + pixel_data_size;

    ico.extend_from_slice(&image_size.to_le_bytes()); // Image data size
    ico.extend_from_slice(&22u32.to_le_bytes()); // Offset to image data (6 + 16 = 22)

    // BITMAPINFOHEADER (40 bytes)
    ico.extend_from_slice(&40u32.to_le_bytes()); // Header size
    ico.extend_from_slice(&(size as i32).to_le_bytes()); // Width
    ico.extend_from_slice(&((size * 2) as i32).to_le_bytes()); // Height (doubled for ICO)
    ico.extend_from_slice(&1u16.to_le_bytes()); // Planes
    ico.extend_from_slice(&32u16.to_le_bytes()); // Bits per pixel
    ico.extend_from_slice(&0u32.to_le_bytes()); // Compression: none
    ico.extend_from_slice(&pixel_data_size.to_le_bytes()); // Image size
    ico.extend_from_slice(&0u32.to_le_bytes()); // X pixels per meter
    ico.extend_from_slice(&0u32.to_le_bytes()); // Y pixels per meter
    ico.extend_from_slice(&0u32.to_le_bytes()); // Colors used
    ico.extend_from_slice(&0u32.to_le_bytes()); // Important colors

    // Pixel data (bottom-up, BGRA)
    for y in (0..size).rev() {
        for x in 0..size {
            let idx = ((y * size + x) * 4) as usize;
            ico.push(bgra[idx]);     // B
            ico.push(bgra[idx + 1]); // G
            ico.push(bgra[idx + 2]); // R
            ico.push(bgra[idx + 3]); // A
        }
    }

    ico
}
