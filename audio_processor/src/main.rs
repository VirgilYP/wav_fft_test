use audio_analysis_tool::plot_frequency_spectrum_with_warning;
use serde_json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let audio_file = "/home/lyp/wav_fft_test/wav/good.wav";
    let warning_offset = 30.0;
    let freq_range_low = 1000.0;
    let freq_range_high = 4000.0;

    match plot_frequency_spectrum_with_warning(audio_file, warning_offset, (freq_range_low, freq_range_high)) {
        Ok(json_result) => {
            println!("{}", json_result);

            // 解析 JSON 结果
            let result: serde_json::Value = serde_json::from_str(&json_result)?;
            println!("Peak value: {}", result["peak_value"]);
            println!("Peak frequency: {}", result["peak_frequency"]);
            println!("Warning: {}", result["warning"]);

            // 保存 base64 编码的图像
            let spectrum_image_base64 = result["spectrum_image_base64"].as_str().unwrap();
            let spectrum_image_data = base64::decode(spectrum_image_base64)?;
            std::fs::write("spectrum_image.png", spectrum_image_data)?;

            println!("Image saved as spectrum_image.png");
        },
        Err(e) => eprintln!("Error: {}", e),
    }

    Ok(())
}
