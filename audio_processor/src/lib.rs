use audrey::read::Reader;
use rustfft::num_complex::Complex;
use rustfft::FftPlanner;
use serde::Serialize;
use jni::JNIEnv;
use jni::objects::{JClass, JString};
use jni::sys::jstring;

#[derive(Serialize)]
struct AudioAnalysisResult {
    peak_value: f32,
    peak_frequency: f32,
    warning: bool,
}

#[no_mangle]
pub extern "system" fn Java_com_example_audioanalysis_AudioAnalysis_plotFrequencySpectrumWithWarning(
    env: JNIEnv,
    _class: JClass,
    audio_file: JString,
    warning_offset: f32,
    freq_range_low: f32,
    freq_range_high: f32,
) -> jstring {
    let audio_file: String = env.get_string(audio_file).expect("Couldn't get java string!").into();
    let freq_range = (freq_range_low, freq_range_high);

    let result = plot_frequency_spectrum_with_warning(&audio_file, warning_offset, freq_range);

    match result {
        Ok(json_result) => {
            let output = env.new_string(json_result).expect("Couldn't create java string!");
            output.into_inner()
        },
        Err(_) => {
            let error_message = env.new_string("Error processing audio file").expect("Couldn't create java string!");
            error_message.into_inner()
        }
    }
}

pub fn plot_frequency_spectrum_with_warning(audio_file: &str, warning_offset: f32, freq_range: (f32, f32)) -> Result<String, Box<dyn std::error::Error>> {
    let mut reader = Reader::open(audio_file)?;
    let desc = reader.description();
    // println!("WAV file format: {:?}", desc);

    let sample_rate = desc.sample_rate() as f32;
    let samples: Vec<f32> = reader.samples::<f32>().map(|s| s.unwrap()).collect();

    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(samples.len());
    let mut buffer: Vec<Complex<f32>> = samples.iter().map(|&s| Complex::new(s, 0.0)).collect();
    fft.process(&mut buffer);

    let xf: Vec<f32> = (0..buffer.len() / 2)
        .map(|i| i as f32 * sample_rate / buffer.len() as f32)
        .collect();

    let yf: Vec<f32> = buffer.iter()
        .take(buffer.len() / 2)
        .map(|c| 2.0 * c.norm() / samples.len() as f32)
        .collect();

    let max_value = i16::MAX as f32;
    let yf_dbfs: Vec<f32> = yf.iter()
        .map(|&y| 20.0 * (y / max_value).log10())
        .collect();

    let mut yf_dbfs_sorted = yf_dbfs.clone();
    yf_dbfs_sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median = yf_dbfs_sorted[yf_dbfs_sorted.len() / 2];
    let baseline_offset = -median;

    let yf_dbfs_adjusted: Vec<f32> = yf_dbfs.iter()
        .map(|&y| y + baseline_offset)
        .collect();

    let freq_mask: Vec<bool> = xf.iter()
        .map(|&f| f >= freq_range.0 && f <= freq_range.1)
        .collect();
    let xf_filtered: Vec<f32> = xf.iter()
        .zip(freq_mask.iter())
        .filter(|(_, &mask)| mask)
        .map(|(&f, _)| f)
        .collect();
    let yf_dbfs_filtered: Vec<f32> = yf_dbfs_adjusted.iter()
        .zip(freq_mask.iter())
        .filter(|(_, &mask)| mask)
        .map(|(&y, _)| y)
        .collect();

    let peak_value = yf_dbfs_filtered.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let peak_freq = xf_filtered[yf_dbfs_filtered.iter().position(|&y| y == peak_value).unwrap()];

    let warning_level_adjusted = warning_offset;
    let warning = peak_value > warning_level_adjusted;

    if warning {
        // println!("Warning: Peak value {:.2} dB exceeds warning level of {:.2} dB in the range {}Hz to {}Hz", peak_value, warning_level_adjusted, freq_range.0, freq_range.1);
    }

    let result = AudioAnalysisResult {
        peak_value,
        peak_frequency: peak_freq,
        warning,
    };

    let json_result = serde_json::to_string(&result)?;
    Ok(json_result)
}
