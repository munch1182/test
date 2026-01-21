pub struct AmplitudeNoiseDetector {
    background_noise_level_db: f64,
    is_calibrated: bool,
    calibration_frames: usize,
    calibration_sum: f64,
    consecutive_voice_frames: usize,
    consecutive_silence_frames: usize,
    
    // 噪声稳定性检测相关字段
    noise_stability_frames: usize,
    last_noise_levels: Vec<f64>,
    is_stable_noise: bool,
    
    // 语音活动历史记录
    voice_activity_history: Vec<bool>,
    
    // 状态稳定性
    last_state: bool,
    state_stability_counter: usize,
}

// 常量配置
const MIN_VOICE_FRAMES: usize = 10;
const MIN_SILENCE_FRAMES: usize = 20;
const ADAPTIVE_ALPHA: f64 = 0.995;
const NOISE_STABILITY_WINDOW: usize = 20;
const VOICE_HISTORY_WINDOW: usize = 5;
const STATE_STABILITY_THRESHOLD: usize = 3;

impl AmplitudeNoiseDetector {
    pub fn new() -> Self {
        AmplitudeNoiseDetector {
            background_noise_level_db: 30.0,
            is_calibrated: false,
            calibration_frames: 0,
            calibration_sum: 0.0,
            consecutive_voice_frames: 0,
            consecutive_silence_frames: 0,
            
            noise_stability_frames: 0,
            last_noise_levels: Vec::with_capacity(NOISE_STABILITY_WINDOW),
            is_stable_noise: false,
            
            voice_activity_history: Vec::with_capacity(VOICE_HISTORY_WINDOW),
            last_state: false,
            state_stability_counter: 0,
        }
    }

    pub fn reset(&mut self) {
        self.background_noise_level_db = 30.0;
        self.is_calibrated = false;
        self.calibration_frames = 0;
        self.calibration_sum = 0.0;
        self.consecutive_voice_frames = 0;
        self.consecutive_silence_frames = 0;
        
        self.noise_stability_frames = 0;
        self.last_noise_levels.clear();
        self.is_stable_noise = false;
        
        self.voice_activity_history.clear();
        self.last_state = false;
        self.state_stability_counter = 0;
    }

    pub fn process_audio_frame(&mut self, bytes: &[i16]) -> (bool, f64, f64, f64) {
        let amplitude = calculate_amplitude(bytes);
        let db = calculate_db(amplitude);

        self.update_background_noise(db);
        let has_voice = self.detect_voice(amplitude, db, bytes);
        
        // 应用状态稳定性处理
        let stable_has_voice = self.apply_state_stabilization(has_voice);

        (stable_has_voice, amplitude, db, self.background_noise_level_db)
    }

    fn update_background_noise(&mut self, db: f64) {
        if !self.is_calibrated {
            // 增加校准帧数以获得更稳定的初始值
            if self.calibration_frames < 30 {
                self.calibration_sum += db;
                self.calibration_frames += 1;

                if self.calibration_frames == 30 {
                    self.background_noise_level_db = self.calibration_sum / 30.0;
                    self.is_calibrated = true;
                }
            }
        } else {
            // 更新噪声水平历史
            self.last_noise_levels.push(db);
            if self.last_noise_levels.len() > NOISE_STABILITY_WINDOW {
                self.last_noise_levels.remove(0);
            }

            // 检测噪声稳定性
            if self.last_noise_levels.len() == NOISE_STABILITY_WINDOW {
                let mean = self.last_noise_levels.iter().sum::<f64>() / NOISE_STABILITY_WINDOW as f64;
                let variance = self.last_noise_levels.iter()
                    .map(|&x| (x - mean).powi(2))
                    .sum::<f64>() / NOISE_STABILITY_WINDOW as f64;
                
                // 如果方差小于阈值，认为噪声稳定
                self.is_stable_noise = variance < 4.0;
            }

            // 根据噪声稳定性调整更新策略
            let update_threshold = if self.is_stable_noise { 8.0 } else { 10.0 };
            let alpha = if self.is_stable_noise { ADAPTIVE_ALPHA } else { 0.98 };

            // 仅在当前帧为静音且接近背景噪声时才更新
            let voice_likely = db > self.background_noise_level_db + update_threshold;
            if !voice_likely {
                self.background_noise_level_db =
                    self.background_noise_level_db * alpha + db * (1.0 - alpha);
            }
        }
    }

    fn detect_voice(&mut self, amplitude: f64, db: f64, bytes: &[i16]) -> bool {
        let zero_cross = calculate_zero_crossing_rate(bytes);
        let spectral_flux = calculate_spectral_flux(bytes);
        let energy_concentration = calculate_energy_concentration(bytes);

        // 基于噪声稳定性的动态阈值调整
        let (db_threshold, amplitude_threshold) = if self.is_stable_noise {
            (
                self.background_noise_level_db + 12.0,
                250.0 + (self.background_noise_level_db * 8.0)
            )
        } else {
            (
                self.background_noise_level_db + 18.0,
                350.0 + (self.background_noise_level_db * 12.0)
            )
        };

        // 权重系统：核心条件权重更高
        let mut score = 0;
        
        // 核心条件：分贝和振幅是关键指标（各2分）
        if db > db_threshold { score += 2; }
        if amplitude > amplitude_threshold { score += 2; }
        
        // 辅助条件：过零率、频谱变化、能量集中度（各1分）
        if (0.02..=0.7).contains(&zero_cross) { score += 1; }
        if spectral_flux > 8.0 { score += 1; }
        if energy_concentration > 0.75 { score += 1; }
        
        // 基于噪声稳定性的动态得分阈值
        let required_score = if self.is_stable_noise { 5 } else { 4 };
        let is_voice_frame = score >= required_score;

        // 更新历史状态用于平滑
        self.voice_activity_history.push(is_voice_frame);
        if self.voice_activity_history.len() > VOICE_HISTORY_WINDOW {
            self.voice_activity_history.remove(0);
        }

        // 历史投票平滑：使用最近N帧的多数状态
        let voice_history_count = self.voice_activity_history.iter()
            .filter(|&&x| x)
            .count();
        let smoothed_voice = if self.voice_activity_history.len() == VOICE_HISTORY_WINDOW {
            voice_history_count > (VOICE_HISTORY_WINDOW / 2)
        } else {
            // 历史数据不足时，直接使用当前帧结果
            is_voice_frame
        };

        // 更新连续帧计数
        if smoothed_voice {
            self.consecutive_voice_frames += 1;
            self.consecutive_silence_frames = 0;
        } else {
            self.consecutive_silence_frames += 1;
            self.consecutive_voice_frames = 0;
        }

        // 最终判断：添加滞后效果防止状态抖动
        if smoothed_voice {
            // 开始语音检测需要满足连续帧数
            self.consecutive_voice_frames >= MIN_VOICE_FRAMES
        } else {
            // 结束语音检测需要更多证据，防止短时停顿误判
            self.consecutive_silence_frames >= MIN_SILENCE_FRAMES
        }
    }

    fn apply_state_stabilization(&mut self, current_state: bool) -> bool {
        // 状态稳定性处理：减少状态抖动
        if current_state == self.last_state {
            self.state_stability_counter += 1;
        } else {
            self.state_stability_counter = 0;
        }

        // 只有状态稳定一定帧数后才切换
        if self.state_stability_counter >= STATE_STABILITY_THRESHOLD {
            self.last_state = current_state;
            current_state
        } else {
            // 保持之前的状态直到稳定
            self.last_state
        }
    }
}

// 工具函数
fn calculate_amplitude(bytes: &[i16]) -> f64 {
    if bytes.is_empty() {
        return 0.0;
    }

    let sum_squares: f64 = bytes
        .iter()
        .map(|&sample| {
            let sample_f64 = sample as f64;
            sample_f64 * sample_f64
        })
        .sum();

    (sum_squares / bytes.len() as f64).sqrt()
}

fn calculate_db(amplitude: f64) -> f64 {
    if amplitude == 0.0 {
        return f64::NEG_INFINITY;
    }

    let max_amplitude = 32767.0;
    let ratio = amplitude / max_amplitude;
    20.0 * ratio.log10()
}

fn calculate_zero_crossing_rate(bytes: &[i16]) -> f64 {
    if bytes.len() < 2 {
        return 0.0;
    }

    let mut zero_crossings = 0;
    for i in 1..bytes.len() {
        if (bytes[i - 1] as i32) * (bytes[i] as i32) < 0 {
            zero_crossings += 1;
        }
    }

    zero_crossings as f64 / (bytes.len() - 1) as f64
}

fn calculate_spectral_flux(bytes: &[i16]) -> f64 {
    if bytes.len() < 2 {
        return 0.0;
    }

    // 添加汉宁窗减少边缘效应
    let mut flux = 0.0;
    let n = bytes.len();
    
    for i in 1..n {
        let window = 0.5 * (1.0 - ((2.0 * std::f64::consts::PI * i as f64) / (n as f64 - 1.0)).cos());
        let diff = (bytes[i] as f64 - bytes[i - 1] as f64).abs() * window;
        flux += diff;
    }

    flux / (n - 1) as f64
}

fn calculate_energy_concentration(bytes: &[i16]) -> f64 {
    if bytes.is_empty() {
        return 0.0;
    }

    // 使用前25%的能量计算集中度
    let mut values: Vec<f64> = bytes.iter().map(|&x| (x as f64).abs()).collect();
    values.sort_by(|a, b| b.partial_cmp(a).unwrap());

    let top_count = ((bytes.len() as f64) * 0.25).ceil() as usize;
    let total_energy: f64 = values.iter().sum();
    
    if total_energy < f64::EPSILON {
        return 0.0;
    }

    let top_energy: f64 = values.iter().take(top_count).sum();
    top_energy / total_energy
}