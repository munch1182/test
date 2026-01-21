mod detector;

use crate::detector::AmplitudeNoiseDetector;
use jni::JNIEnv;
use jni::objects::{JClass, JPrimitiveArray};
use jni::sys::jint;
use log::info;
use std::sync::{LazyLock, Mutex};

static DETECTOR: Mutex<LazyLock<AmplitudeNoiseDetector>> = Mutex::new(LazyLock::new(|| {
    init_android_log();
    AmplitudeNoiseDetector::new()
}));

fn init_android_log() {
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Debug)
            .with_tag("AmplitudeNoiseDetector"),
    );
}

/// 处理音频帧
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_munch1182_android_lib_helper_audio_AmplitudeNoiseDetector_processAudioFrame(
    env: JNIEnv,
    _class: JClass,
    audio_data: jni::sys::jarray,
) -> jint {
    let jarray = unsafe { JPrimitiveArray::from_raw(audio_data) };
    println!("audio_data: 111");
    if let Ok(len) = env.get_array_length(&jarray) {
        let mut buf = vec![0i16; len as usize];
        if env.get_short_array_region(jarray, 0, &mut buf).is_ok() {
            if let Ok(mut detector) = DETECTOR.lock() {
                let (has_voice, amplitude, db, background_db) = detector.process_audio_frame(&buf);
                info!(
                    "has_voice: {has_voice}, amplitude: {amplitude}, db: {db}, background_db: {background_db}"
                );
                return if has_voice { 1 } else { -1 };
            }
            info!("get lock fail");
        }
    }
    info!("get array length failed");
    0
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_munch1182_android_lib_helper_audio_AmplitudeNoiseDetector_reset(
    _env: JNIEnv,
    _class: JClass,
) -> jint {
    if let Ok(mut detector) = DETECTOR.lock() {
        detector.reset();
        return -1;
    }
    1
}
