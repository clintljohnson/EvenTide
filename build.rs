//! Embeds the application icon as resource ID 1 so Explorer can show it.
//!
//! The frames are the premultiplied BGRA squares in `assets`. Windows reads
//! `RT_GROUP_ICON` from the PE; an in-memory `HICON` does not change the file.

use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    generate_places(&manifest, &out_dir);
    // Premultiplied BGRA frames scaled from `Icons\AppIcon-v3.png`.
    let sizes = [16usize, 24, 48, 72, 256];
    let mut images = Vec::new();
    for side in sizes {
        let path = manifest.join("assets").join(format!("icon-{side}.bgra"));
        println!("cargo:rerun-if-changed={}", path.display());
        let pixels = fs::read(&path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()));
        images.push(icon_dib(&pixels, side));
    }
    let ico = write_ico(&images, &sizes);
    let ico_path = manifest.join("assets").join("appicon.ico");
    fs::write(&ico_path, &ico).unwrap_or_else(|err| panic!("write {}: {err}", ico_path.display()));
    let res_path = out_dir.join("appicon.res");
    fs::write(&res_path, write_res(&images))
        .unwrap_or_else(|err| panic!("write {}: {err}", res_path.display()));
    println!("cargo:rustc-link-arg-bins={}", res_path.display());
}

fn generate_places(manifest: &std::path::Path, out_dir: &std::path::Path) {
    let path = manifest.join("assets").join("places.tsv");
    println!("cargo:rerun-if-changed={}", path.display());
    let text = fs::read_to_string(&path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()));
    let mut out = String::from(
        "// Generated from assets/places.tsv. Natural Earth populated places, public domain.\n",
    );
    out.push_str("pub static PLACES: &[Place] = &[\n");
    for (line_number, line) in text.lines().enumerate() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut parts = line.split('\t');
        let name = parts.next().unwrap_or_else(|| panic!("name on line {line_number}"));
        let country = parts
            .next()
            .unwrap_or_else(|| panic!("country on line {line_number}"));
        let latitude: i32 = parts
            .next()
            .unwrap_or_else(|| panic!("latitude on line {line_number}"))
            .parse()
            .unwrap_or_else(|err| panic!("latitude on line {line_number}: {err}"));
        let longitude: i32 = parts
            .next()
            .unwrap_or_else(|| panic!("longitude on line {line_number}"))
            .parse()
            .unwrap_or_else(|err| panic!("longitude on line {line_number}: {err}"));
        let country_name = parts
            .next()
            .unwrap_or_else(|| panic!("country name on line {line_number}"));
        let region = parts.next().unwrap_or("");
        out.push_str(&format!(
            "    Place {{ name: \"{}\", country: \"{}\", country_name: \"{}\", region: \"{}\", latitude_hundredths: {latitude}, longitude_hundredths: {longitude} }},\n",
            rust_string(name),
            rust_string(country),
            rust_string(country_name),
            rust_string(region),
        ));
    }
    out.push_str("];\n");
    let dest = out_dir.join("places.rs");
    fs::write(&dest, out).unwrap_or_else(|err| panic!("write {}: {err}", dest.display()));
}

fn rust_string(text: &str) -> String {
    text.replace('\\', "\\\\").replace('"', "\\\"")
}

fn icon_dib(pixels: &[u8], side: usize) -> Vec<u8> {
    assert_eq!(
        pixels.len(),
        side * side * 4,
        "icon-{side}.bgra has the wrong length"
    );
    let xor_bytes = side * side * 4;
    let mask_stride = (side + 31) / 32 * 4;
    let mask_bytes = mask_stride * side;
    let mut dib = vec![0u8; 40 + xor_bytes + mask_bytes];
    dib[0..4].copy_from_slice(&40u32.to_le_bytes());
    dib[4..8].copy_from_slice(&(side as i32).to_le_bytes());
    dib[8..12].copy_from_slice(&((side as i32) * 2).to_le_bytes());
    dib[12..14].copy_from_slice(&1u16.to_le_bytes());
    dib[14..16].copy_from_slice(&32u16.to_le_bytes());
    dib[20..24].copy_from_slice(&((xor_bytes + mask_bytes) as u32).to_le_bytes());
    let row = side * 4;
    for y in 0..side {
        let src = (side - 1 - y) * row;
        let dst = 40 + y * row;
        for x in 0..side {
            let s = src + x * 4;
            let d = dst + x * 4;
            let (b, g, r, a) = (pixels[s], pixels[s + 1], pixels[s + 2], pixels[s + 3]);
            let (b, g, r) = unpremultiply(b, g, r, a);
            dib[d] = b;
            dib[d + 1] = g;
            dib[d + 2] = r;
            dib[d + 3] = a;
            if a == 0 {
                let mask_row = 40 + xor_bytes + y * mask_stride;
                dib[mask_row + x / 8] |= 0x80 >> (x % 8);
            }
        }
    }
    dib
}

fn unpremultiply(b: u8, g: u8, r: u8, a: u8) -> (u8, u8, u8) {
    if a == 0 || a == 255 {
        return (b, g, r);
    }
    let scale = |c: u8| ((u16::from(c) * 255) / u16::from(a)).min(255) as u8;
    (scale(b), scale(g), scale(r))
}

fn write_ico(images: &[Vec<u8>], sizes: &[usize]) -> Vec<u8> {
    let count = images.len() as u16;
    let mut file = Vec::new();
    file.extend_from_slice(&0u16.to_le_bytes());
    file.extend_from_slice(&1u16.to_le_bytes());
    file.extend_from_slice(&count.to_le_bytes());
    let dir = 6 + images.len() * 16;
    let mut offset = dir;
    for (image, side) in images.iter().zip(sizes) {
        let dim = if *side >= 256 { 0u8 } else { *side as u8 };
        file.push(dim);
        file.push(dim);
        file.push(0);
        file.push(0);
        file.extend_from_slice(&1u16.to_le_bytes());
        file.extend_from_slice(&32u16.to_le_bytes());
        file.extend_from_slice(&(image.len() as u32).to_le_bytes());
        file.extend_from_slice(&(offset as u32).to_le_bytes());
        offset += image.len();
    }
    for image in images {
        file.extend_from_slice(image);
    }
    file
}

fn write_res(images: &[Vec<u8>]) -> Vec<u8> {
    let mut res = Vec::new();
    // Resource compilers start the file with an empty 32-byte entry.
    res.extend_from_slice(&[0, 0, 0, 0, 0x20, 0, 0, 0]);
    res.extend_from_slice(&[0xFF, 0xFF, 0, 0, 0xFF, 0xFF, 0, 0]);
    res.extend_from_slice(&[0u8; 16]);
    for (index, image) in images.iter().enumerate() {
        push_resource(&mut res, 3, (index + 1) as u16, image);
    }
    push_resource(&mut res, 14, 1, &group_icon(images));
    res
}

fn group_icon(images: &[Vec<u8>]) -> Vec<u8> {
    let mut data = Vec::new();
    data.extend_from_slice(&0u16.to_le_bytes());
    data.extend_from_slice(&1u16.to_le_bytes());
    data.extend_from_slice(&(images.len() as u16).to_le_bytes());
    for (index, image) in images.iter().enumerate() {
        let side = icon_side(image);
        let dim = if side >= 256 { 0u8 } else { side as u8 };
        data.push(dim);
        data.push(dim);
        data.push(0);
        data.push(0);
        data.extend_from_slice(&1u16.to_le_bytes());
        data.extend_from_slice(&32u16.to_le_bytes());
        data.extend_from_slice(&(image.len() as u32).to_le_bytes());
        data.extend_from_slice(&((index + 1) as u16).to_le_bytes());
    }
    data
}

fn icon_side(image: &[u8]) -> usize {
    let width = i32::from_le_bytes(image[4..8].try_into().unwrap());
    width as usize
}

fn push_resource(out: &mut Vec<u8>, kind: u16, name: u16, data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_le_bytes());
    out.extend_from_slice(&32u32.to_le_bytes());
    out.extend_from_slice(&0xFFFFu16.to_le_bytes());
    out.extend_from_slice(&kind.to_le_bytes());
    out.extend_from_slice(&0xFFFFu16.to_le_bytes());
    out.extend_from_slice(&name.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&0x0030u16.to_le_bytes());
    out.extend_from_slice(&0x0409u16.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(data);
    while out.len() % 4 != 0 {
        out.push(0);
    }
}
