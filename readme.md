

# 在 Android 项目中集成音频分析动态库的教程

本教程将指导你如何在 Android 项目中集成和使用一个已经编译好的音频分析动态库（`.so` 文件）。

## 前提条件

- 已安装 Android Studio
- 已安装 Android NDK

## 步骤

### 1. 创建 Android 项目

如果你还没有 Android 项目，可以使用 Android Studio 创建一个新的项目。

### 2. 添加动态库

将提供的 `libaudio_analysis_tool.so` 文件复制到 Android 项目的 `src/main/jniLibs/arm64-v8a/` 文件夹中：

```
src/main/jniLibs/arm64-v8a/libaudio_analysis_tool.so
```

### 3. 配置 `CMakeLists.txt`

在 `src/main/cpp` 文件夹中创建 `CMakeLists.txt` 文件（如果还没有），并添加以下内容：

```cmake
cmake_minimum_required(VERSION 3.4.1)

add_library(audio_analysis_tool SHARED IMPORTED)
set_target_properties(audio_analysis_tool PROPERTIES IMPORTED_LOCATION
    ${CMAKE_SOURCE_DIR}/../../../../src/main/jniLibs/arm64-v8a/libaudio_analysis_tool.so)

find_library(log-lib log)

target_link_libraries(audio_analysis_tool ${log-lib})
```

### 4. 配置 `build.gradle`

在 `app` 目录下的 `build.gradle` 文件中，添加对 JNI 和 CMake 的支持：

```groovy
android {
    ...
    defaultConfig {
        ...
        externalNativeBuild {
            cmake {
                cppFlags ""
            }
        }
        ndk {
            abiFilters "arm64-v8a"
        }
    }
    externalNativeBuild {
        cmake {
            path "src/main/cpp/CMakeLists.txt"
        }
    }
}
```

### 5. 创建 JNI 接口

在 `com.example.audioanalysis` 包中创建一个新的 Kotlin 类 `AudioAnalysis.kt`：

```kotlin
package com.example.audioanalysis

class AudioAnalysis {
    companion object {
        init {
            System.loadLibrary("audio_analysis_tool")
        }

        @JvmStatic
        external fun plotFrequencySpectrumWithWarning(
            audioFile: String,
            warningOffset: Float,
            freqRangeLow: Float,
            freqRangeHigh: Float
        ): String
    }
}
```

### 6. 调用动态库函数

在你的 Android 应用中调用动态库中的函数。例如，在某个 Activity 中：

```kotlin
import com.example.audioanalysis.AudioAnalysis

fun analyzeAudio() {
    val audioFile = "/path/to/your/audio/file.wav"
    val warningOffset = 30.0f
    val freqRangeLow = 1000.0f
    val freqRangeHigh = 4000.0f

    val resultJson = AudioAnalysis.plotFrequencySpectrumWithWarning(audioFile, warningOffset, freqRangeLow, freqRangeHigh)
    println(resultJson)
}
```

### 7. 编译和运行

确保你的项目配置正确，然后在 Android Studio 中编译并运行你的项目。

## 示例项目结构

```
MyAndroidApp/
├── app/
│   ├── src/
│   │   ├── main/
│   │   │   ├── java/
│   │   │   │   └── com/
│   │   │   │       └── example/
│   │   │   │           └── audioanalysis/
│   │   │   │               └── AudioAnalysis.kt
│   │   │   ├── jniLibs/
│   │   │   │   └── arm64-v8a/
│   │   │   │       └── libaudio_analysis_tool.so
│   │   │   └── cpp/
│   │   │       └── CMakeLists.txt
│   └── build.gradle
├── build.gradle
└── settings.gradle
```

这样，你就可以在 Android 项目中调用音频分析工具了。如果有任何问题，请确保你的项目配置正确，并检查日志以获取更多信息。



