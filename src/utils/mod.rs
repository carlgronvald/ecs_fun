


#[derive(Debug, Clone, Copy)]
pub enum ColorFormat {
    RGB,
    RGBA,
    /// 16 bit depth format
    D,
}

impl ColorFormat {
    /// Returns the GL texture type, but not the internal format.
    pub fn gl_format(&self) -> u32 {
        match self {
            ColorFormat::RGB => gl::RGB,
            ColorFormat::RGBA => gl::RGBA,
            ColorFormat::D => gl::DEPTH_COMPONENT,
        }
    }
}



use std::path::Path;


/// Contains the info you get when you load a png
pub struct PngData {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
    pub format: ColorFormat,
}

#[derive(Debug)]
pub enum PngLoadError {
    InvalidFormat,
    Decoding(png::DecodingError),
    IO(std::io::Error),
}

/// TODO: MAKE THIS GIVE ERRORS CORRECTLY
pub fn read_png(path: &Path) -> Result<PngData, PngLoadError> {
    use std::fs::File;

    let file = match File::open(path) {
        Ok(f) => f,
        Err(error) => return Err(PngLoadError::IO(error)),
    };

    let decoder = png::Decoder::new(file);

    let (info, mut reader) = match decoder.read_info() {
        Ok(ir) => ir,
        Err(error) => return Err(PngLoadError::Decoding(error)),
    };

    // Allocate the output buffer.
    let mut buf = vec![0; info.buffer_size()];
    // Read the next frame. An APNG might contain multiple frames.
    match reader.next_frame(&mut buf) {
        Ok(_) => (),
        Err(error) => return Err(PngLoadError::Decoding(error)),
    };

    let format = match info.color_type {
        png::ColorType::RGB => ColorFormat::RGB,
        png::ColorType::RGBA => ColorFormat::RGBA,
        _ => return Err(PngLoadError::InvalidFormat),
    };

    Ok(PngData {
        width: info.width,
        height: info.height,
        data: buf,
        format,
    })
}

use std::fs::{self, DirEntry};
use std::io;

#[derive(Debug)]
pub enum VisitDirError {
    IOError { error: io::Error },
    NonDirError,
}
/// Recursively collects the dir entries below the chosen directory, bundled with a forward slash separated path from the starting dir to that entry (starting with a /).
/// If you want to, you can make a prefix for the forward slash separated paths, by handing it an inner_path.
pub fn dir_entries(dir: &Path, inner_path: &str) -> Result<Vec<(DirEntry, String)>, VisitDirError> {
    if dir.is_dir() {
        let mut result = Vec::new();
        let iterator = match fs::read_dir(dir) {
            Ok(i) => i,
            Err(error) => return Err(VisitDirError::IOError { error }),
        };
        for entry in iterator {
            let entry = match entry {
                Ok(e) => e,
                Err(error) => return Err(VisitDirError::IOError { error }),
            };
            let path = entry.path();
            let forward_slash_path = &format!(
                "{}/{}",
                inner_path,
                path.file_name().unwrap().to_str().unwrap()
            );

            if path.is_dir() {
                let mut append = dir_entries(&path, forward_slash_path)?;
                result.append(&mut append);
            } else {
                result.push((entry, forward_slash_path.to_owned()));
            }
        }
        Ok(result)
    } else {
        Err(VisitDirError::NonDirError)
    }
}