use std::fs::File;
use std::path::Path;
use std::thread;
use std::time::Duration;
use symphonia::core::audio::{AudioBufferRef, Signal};
use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

/// Callback type for progress updates (progress: 0.0 to 1.0)
pub type ProgressCallback = Box<dyn Fn(f32) + Send>;

/// Generate waveform data from an audio file
/// Returns a vector of normalized amplitude values (0.0 to 1.0)
/// The number of samples is reduced based on the target_samples parameter
pub fn generate_waveform(file_path: &str, target_samples: usize) -> Result<Vec<f32>, String> {
    generate_waveform_with_progress(file_path, target_samples, None)
}

/// Generate waveform data from an audio file with progress callback
pub fn generate_waveform_with_progress(
    file_path: &str,
    target_samples: usize,
    progress_callback: Option<ProgressCallback>,
) -> Result<Vec<f32>, String> {
    let path = Path::new(file_path);

    // Open the media source
    let file = File::open(path)
        .map_err(|e| format!("Failed to open audio file: {}", e))?;

    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    // Create a hint to help the format registry guess the format
    let mut hint = Hint::new();
    if let Some(extension) = path.extension() {
        if let Some(ext_str) = extension.to_str() {
            hint.with_extension(ext_str);
        }
    }

    // Probe the media source
    let format_opts = FormatOptions::default();
    let metadata_opts = MetadataOptions::default();

    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &format_opts, &metadata_opts)
        .map_err(|e| format!("Failed to probe audio file: {}", e))?;

    let mut format = probed.format;

    // Find the first audio track
    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .ok_or_else(|| "No audio track found".to_string())?;

    let track_id = track.id;

    // Create a decoder for the track
    let dec_opts = DecoderOptions::default();
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &dec_opts)
        .map_err(|e| format!("Failed to create decoder: {}", e))?;

    // Collect all audio samples
    let mut all_samples: Vec<f32> = Vec::new();
    let mut packet_count = 0u64;
    let mut last_progress = 0.0f32;

    // Decode packets
    loop {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(_) => break,
        };

        // Skip packets that don't belong to the selected track
        if packet.track_id() != track_id {
            continue;
        }

        // Decode the packet
        match decoder.decode(&packet) {
            Ok(decoded) => {
                // Extract samples from the decoded audio buffer
                let samples = extract_samples(&decoded);
                all_samples.extend(samples);
                
                // Update progress (decode phase: 0-80%)
                packet_count += 1;
                if packet_count % 100 == 0 {
                    let progress = (packet_count as f32 / 5000.0).min(0.8);
                    if progress > last_progress + 0.02 {
                        last_progress = progress;
                        if let Some(ref callback) = progress_callback {
                            callback(progress);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Decode error: {}", e);
                continue;
            }
        }
    }

    if all_samples.is_empty() {
        return Err("No audio samples extracted".to_string());
    }

    // Report progress: 80% - starting downsample
    if let Some(ref callback) = progress_callback {
        callback(0.8);
        // 短暂延迟确保事件能被发送到前端
        thread::sleep(Duration::from_millis(10));
    }

    // Downsample to target number of samples
    let waveform = downsample_and_normalize_with_progress(&all_samples, target_samples, progress_callback.as_ref());

    // Report progress: 100% - complete
    if let Some(ref callback) = progress_callback {
        callback(1.0);
        // 短暂延迟确保100%进度事件能被前端接收后再返回
        thread::sleep(Duration::from_millis(50));
    }

    Ok(waveform)
}

/// Extract samples from an audio buffer and convert to mono f32
fn extract_samples(decoded: &AudioBufferRef) -> Vec<f32> {
    match decoded {
        AudioBufferRef::F32(buf) => {
            // Convert to mono by averaging channels
            let num_channels = buf.spec().channels.count();
            let num_frames = buf.frames();
            let mut mono_samples = Vec::with_capacity(num_frames);

            for frame_idx in 0..num_frames {
                let mut sum = 0.0;
                for ch in 0..num_channels {
                    sum += buf.chan(ch)[frame_idx];
                }
                mono_samples.push(sum / num_channels as f32);
            }
            mono_samples
        }
        AudioBufferRef::S32(buf) => {
            // Convert i32 samples to f32
            let num_channels = buf.spec().channels.count();
            let num_frames = buf.frames();
            let mut mono_samples = Vec::with_capacity(num_frames);

            for frame_idx in 0..num_frames {
                let mut sum = 0.0;
                for ch in 0..num_channels {
                    sum += buf.chan(ch)[frame_idx] as f32 / i32::MAX as f32;
                }
                mono_samples.push(sum / num_channels as f32);
            }
            mono_samples
        }
        AudioBufferRef::S16(buf) => {
            // Convert i16 samples to f32
            let num_channels = buf.spec().channels.count();
            let num_frames = buf.frames();
            let mut mono_samples = Vec::with_capacity(num_frames);

            for frame_idx in 0..num_frames {
                let mut sum = 0.0;
                for ch in 0..num_channels {
                    sum += buf.chan(ch)[frame_idx] as f32 / i16::MAX as f32;
                }
                mono_samples.push(sum / num_channels as f32);
            }
            mono_samples
        }
        AudioBufferRef::U8(buf) => {
            // Convert u8 samples to f32
            let num_channels = buf.spec().channels.count();
            let num_frames = buf.frames();
            let mut mono_samples = Vec::with_capacity(num_frames);

            for frame_idx in 0..num_frames {
                let mut sum = 0.0;
                for ch in 0..num_channels {
                    // u8 audio is 0-255, convert to -1.0 to 1.0
                    sum += (buf.chan(ch)[frame_idx] as f32 - 128.0) / 128.0;
                }
                mono_samples.push(sum / num_channels as f32);
            }
            mono_samples
        }
        _ => {
            // For other formats, return empty
            Vec::new()
        }
    }
}

/// Downsample audio data and normalize to 0.0-1.0 range
/// Uses peak amplitude per chunk for better waveform visualization
fn downsample_and_normalize(samples: &[f32], target_samples: usize) -> Vec<f32> {
    downsample_and_normalize_with_progress(samples, target_samples, None)
}

/// Downsample audio data with progress callback
fn downsample_and_normalize_with_progress(
    samples: &[f32],
    target_samples: usize,
    progress_callback: Option<&ProgressCallback>,
) -> Vec<f32> {
    if samples.is_empty() {
        return Vec::new();
    }

    let total_samples = samples.len();

    if total_samples <= target_samples {
        // If we have fewer samples than target, just normalize
        // 报告进度：90%（快速处理阶段）
        if let Some(ref callback) = progress_callback {
            callback(0.9);
            thread::sleep(Duration::from_millis(10));
        }
        return samples.iter().map(|&s| s.abs()).collect();
    }

    let chunk_size = total_samples / target_samples;
    let mut waveform = Vec::with_capacity(target_samples);

    // For each chunk, find the peak amplitude
    for i in 0..target_samples {
        let start = i * chunk_size;
        let end = if i == target_samples - 1 {
            total_samples
        } else {
            (i + 1) * chunk_size
        };

        let chunk = &samples[start..end];
        let peak = chunk.iter().map(|&s| s.abs()).fold(0.0f32, f32::max);
        waveform.push(peak);
        
        // Update progress (downsample phase: 80-100%)
        // 每1000个元素更新一次（约每5%更新一次），确保有足够时间让事件传递到前端
        if let Some(ref callback) = progress_callback {
            if i % 1000 == 0 || i == target_samples - 1 {
                let current_progress = 0.8 + ((i + 1) as f32 / target_samples as f32) * 0.2;
                // 确保最后一个元素时进度接近100%（但不等于100%，因为后面会设置为100%）
                let progress_to_report = if i == target_samples - 1 {
                    0.99 // 设置为99%，让主函数设置为100%
                } else {
                    current_progress.min(0.99) // 限制最大值为99%
                };
                callback(progress_to_report);
                // 短暂延迟确保事件能被发送到前端
                thread::sleep(Duration::from_millis(5));
            }
        }
    }

    waveform
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_downsample() {
        let samples = vec![0.5, 0.8, 0.3, 0.9, 0.1, 0.7];
        let result = downsample_and_normalize(&samples, 3);
        assert_eq!(result.len(), 3);
    }
}
