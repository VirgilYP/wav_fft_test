import numpy as np
import matplotlib.pyplot as plt
from scipy.io import wavfile

def plot_frequency_spectrum_with_warning(audio_file, warning_level=-12):
    # 读取音频文件
    sample_rate, data = wavfile.read(audio_file)

    # 如果音频是立体声，取一个声道数据
    if len(data.shape) > 1:
        data = data[:, 0]

    # 定义FFT窗口大小
    fft_size = 1024

    # 应用汉宁窗
    window = np.hanning(fft_size)

    # 计算FFT
    N = len(data)
    T = 1.0 / sample_rate
    segment = data[:fft_size] * window  # 取音频数据的前1024个样本并应用窗口函数
    yf = np.fft.fft(segment)
    xf = np.fft.fftfreq(fft_size, T)[:fft_size//2]

    # 归一化处理
    yf_normalized = 2.0/fft_size * np.abs(yf[0:fft_size//2])

    # 转换为dBFS
    max_value = np.iinfo(data.dtype).max  # 获取数据类型的最大值，例如32768
    yf_dbfs = 20 * np.log10(yf_normalized / max_value)

    # 找到峰值
    peak_value = np.max(yf_dbfs)
    peak_freq = xf[np.argmax(yf_dbfs)]

    # 检查是否超过警戒线
    warning = peak_value > warning_level

    if warning:
        print(f"Warning: Peak value {peak_value:.2f} dBFS exceeds warning level of {warning_level} dBFS")

    # 绘制频谱图
    plt.figure(figsize=(12, 6))
    plt.plot(xf, yf_dbfs)
    plt.axhline(y=peak_value, color='b', linestyle='--', label=f'Peak: {peak_value:.2f} dBFS at {peak_freq:.2f} Hz')
    plt.axhline(y=warning_level, color='r', linestyle='--', label=f'Warning Line: {warning_level} dBFS')  # 添加警戒线
    plt.grid()
    plt.xlabel('Frequency (Hz)')
    plt.ylabel('Amplitude (dBFS)')
    plt.title('Frequency Spectrum (1024-point FFT with Hanning Window)')
    plt.legend()
    plt.show()

    return "Warning: Peak value exceeds warning level" if warning else "No warning"

# 使用示例
audio_file = './wav/good.wav'
result = plot_frequency_spectrum_with_warning(audio_file)
print(result)
