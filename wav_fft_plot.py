import numpy as np
import matplotlib.pyplot as plt
from scipy.io import wavfile

def plot_frequency_spectrum_with_warning(audio_file, warning_offset=30, freq_range=(1000, 4000)):
    # 读取音频文件
    sample_rate, data = wavfile.read(audio_file)

    # 如果音频是立体声，取一个声道数据
    if len(data.shape) > 1:
        data = data[:, 0]

    # 计算FFT
    N = len(data)
    T = 1.0 / sample_rate
    yf = np.fft.fft(data)
    xf = np.fft.fftfreq(N, T)[:N//2]

    # 归一化处理
    yf_normalized = 2.0/N * np.abs(yf[0:N//2])

    # 转换为dBFS
    max_value = np.iinfo(data.dtype).max  # 获取数据类型的最大值，例如32768
    yf_dbfs = 20 * np.log10(yf_normalized / max_value)

    # 计算基线偏移量（使用中位数）
    baseline_offset = -np.median(yf_dbfs)

    # 应用基线偏移
    yf_dbfs += baseline_offset

    # 限制频率范围
    freq_mask = (xf >= freq_range[0]) & (xf <= freq_range[1])
    xf_filtered = xf[freq_mask]
    yf_dbfs_filtered = yf_dbfs[freq_mask]

    # 找到指定频率范围内的峰值
    peak_value = np.max(yf_dbfs_filtered)
    peak_freq = xf_filtered[np.argmax(yf_dbfs_filtered)]

    # 设置警告线在基线的30 dB处
    warning_level_adjusted = warning_offset  # 基线的30 dB处

    # 检查是否超过警戒线
    warning = peak_value > warning_level_adjusted

    if warning:
        print(f"Warning: Peak value {peak_value:.2f} dB exceeds warning level of {warning_level_adjusted:.2f} dB in the range {freq_range[0]}Hz to {freq_range[1]}Hz")

    # 绘制频谱图
    plt.figure(figsize=(12, 6))
    plt.plot(xf, yf_dbfs, label='Full Spectrum')
    plt.plot(xf_filtered, yf_dbfs_filtered, label=f'Spectrum in {freq_range[0]}-{freq_range[1]} Hz')
    plt.axhline(y=0, color='k', linestyle='-', label='Baseline: 0 dB (adjusted)')  # 添加基线
    plt.axhline(y=peak_value, color='b', linestyle='--', label=f'Peak: {peak_value:.2f} dB at {peak_freq:.2f} Hz (adjusted)')
    plt.axhline(y=warning_level_adjusted, color='r', linestyle='--', label=f'Warning Line: {warning_level_adjusted:.2f} dB above baseline')  # 添加警戒线
    plt.grid()
    plt.xlabel('Frequency (Hz)')
    plt.ylabel('Amplitude (dB)')
    plt.title('Frequency Spectrum (Full FFT)')
    plt.ylim(-200 + baseline_offset, 50 + baseline_offset)  # 设置y轴范围
    plt.legend()
    plt.show()

    return "Warning: Peak value exceeds warning level" if warning else "No warning"

# 使用示例
audio_file = './wav/good.wav'
result = plot_frequency_spectrum_with_warning(audio_file)
print(result)
