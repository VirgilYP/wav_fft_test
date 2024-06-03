use audrey::read::Reader;
use rustfft::num_complex::Complex;
use rustfft::FftPlanner;
use plotters::prelude::*;
use serde::Serialize;
use std::convert::TryInto;
use image::{ImageBuffer, DynamicImage, Rgb};
use base64::encode;

#[derive(Serialize)]
struct AudioAnalysisResult {
    peak_value: f32,
    peak_frequency: f32,
    warning: bool,
    spectrum_image_base64: String,
}

pub fn plot_frequency_spectrum_with_warning(audio_file: &str, warning_offset: f32, freq_range: (f32, f32)) -> Result<String, Box<dyn std::error::Error>> {
    let mut reader = Reader::open(audio_file)?;
    let desc = reader.description();
    println!("WAV file format: {:?}", desc);

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
        println!("Warning: Peak value {:.2} dB exceeds warning level of {:.2} dB in the range {}Hz to {}Hz", peak_value, warning_level_adjusted, freq_range.0, freq_range.1);
    }

    // 绘制频谱图并保存到内存中
    let width = 1280u32;
    let height = 720u32;
    let buffer_size = (width * height * 3) as usize; // 使用3通道缓冲区
    let mut buffer = vec![0u8; buffer_size];

    {
        let root = BitMapBackend::with_buffer(&mut buffer, (width, height)).into_drawing_area();
        root.fill(&WHITE)?; // 确保在每次绘图之前清空绘图区域
        let mut chart = ChartBuilder::on(&root)
            .caption("Frequency Spectrum", ("sans-serif", 50).into_font())
            .margin(10)
            .x_label_area_size(30)
            .y_label_area_size(50) // 增加Y轴标签区域的大小
            .build_cartesian_2d(0f32..(sample_rate / 2.0), -100f32..100f32)?; // 调整Y轴范围

        chart.configure_mesh()
            .x_desc("Frequency (Hz)")
            .y_desc("Amplitude (dB)")
            .axis_desc_style(("sans-serif", 20))
            .label_style(("sans-serif", 15))
            .light_line_style(&TRANSPARENT) // 隐藏次要网格线
            .bold_line_style(ShapeStyle {
                color: RGBColor(200, 200, 200).to_rgba(), // 设置主要网格线的颜色
                filled: true,
                stroke_width: 1,
            })
            .draw()?;

        chart.draw_series(LineSeries::new(
            xf.iter().cloned().zip(yf_dbfs_adjusted.iter().cloned()),
            &BLUE,
        ))?.label("Full Spectrum").legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));

        chart.draw_series(LineSeries::new(
            xf_filtered.iter().cloned().zip(yf_dbfs_filtered.iter().cloned()),
            &RED,
        ))?.label(format!("Spectrum in {}-{} Hz", freq_range.0, freq_range.1)).legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

        chart.draw_series(std::iter::once(PathElement::new(
            vec![(0.0, 0.0), (sample_rate / 2.0, 0.0)],
            &BLACK,
        )))?.label("Baseline: 0 dB (adjusted)").legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLACK));

        chart.draw_series(std::iter::once(PathElement::new(
            vec![(0.0, peak_value), (sample_rate / 2.0, peak_value)],
            &GREEN,
        )))?.label(format!("Peak: {:.2} dB at {:.2} Hz (adjusted)", peak_value, peak_freq)).legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &GREEN));

        chart.draw_series(std::iter::once(PathElement::new(
            vec![(0.0, warning_level_adjusted), (sample_rate / 2.0, warning_level_adjusted)],
            &MAGENTA,
        )))?.label(format!("Warning Line: {:.2} dB above baseline", warning_level_adjusted)).legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &MAGENTA));

        chart.configure_series_labels()
            .background_style(&WHITE.mix(0.8)) // 设置背景样式
            .border_style(&BLACK) // 设置边框样式
            .position(SeriesLabelPosition::UpperRight) // 设置图例的位置为右上角
            .label_font(("sans-serif", 25)) // 设置图例字体大小
            .draw()?;
    }

    // 将缓冲区转换为 PNG 格式并编码为 base64
    let buffer = ImageBuffer::<Rgb<u8>, _>::from_raw(width, height, buffer).unwrap();
    let dynamic_image = DynamicImage::ImageRgb8(buffer);
    let mut png_buffer = vec![];
    dynamic_image.write_to(&mut png_buffer, image::ImageOutputFormat::Png)?;
    let base64_image = encode(&png_buffer);

    let result = AudioAnalysisResult {
        peak_value,
        peak_frequency: peak_freq,
        warning,
        spectrum_image_base64: base64_image,
    };

    let json_result = serde_json::to_string(&result)?;
    Ok(json_result)
}
