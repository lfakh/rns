use image::ImageFormat;
use std::io::Cursor;
use std::path::PathBuf;
use tauri::AppHandle;
use tauri::Manager;
use std::fs;
use audiopus::coder::Encoder;
use audiopus::{Application, SampleRate, Channels};

pub fn process_image_to_avif(input_bytes: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let img = image::load_from_memory(input_bytes)?;
    let mut output_bytes = Vec::new();
    let mut cursor = Cursor::new(&mut output_bytes);
    
    // We can adjust quality here if needed using image::codecs::avif::AvifEncoder
    img.write_to(&mut cursor, ImageFormat::Avif)?;
    
    Ok(output_bytes)
}

pub fn process_audio_to_opus(pcm_data: &[f32]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Standard settings for voice chat
    let mut encoder = Encoder::new(SampleRate::Hz48000, Channels::Mono, Application::Voip)?;
    
    // Opus encoding requires fixed frame sizes. 
    // For 48kHz mono, a 20ms frame is 960 samples.
    let frame_size = 960;
    let mut output_bytes = Vec::new();
    let mut buffer = [0u8; 1024];

    for chunk in pcm_data.chunks_exact(frame_size) {
        let len = encoder.encode_float(chunk, &mut buffer)?;
        output_bytes.extend_from_slice(&buffer[..len]);
    }
    
    Ok(output_bytes)
}

pub fn save_media(app_handle: &AppHandle, data: &[u8], extension: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let app_dir = app_handle.path().app_data_dir()?;
    let media_dir = app_dir.join("media");
    if !media_dir.exists() {
        fs::create_dir_all(&media_dir)?;
    }
    
    let filename = format!("{}.{}", uuid::Uuid::new_v4(), extension);
    let file_path = media_dir.join(filename);
    fs::write(&file_path, data)?;
    
    Ok(file_path)
}
