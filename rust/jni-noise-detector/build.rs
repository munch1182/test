/// 1. 添加目标：rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android // 选择架构
/// 2. 安装ndk，通过androidstudio/SDK Manager/SDK Tools/NDK (Side by side)
/// 3. 配置环境变量ANDROID_HOME和ANDROID_NDK_HOME: 查看：cmd: set android
/// 4. 安装cargo-ndk: cargo install cargo-ndk 用于编译命令使用环境变量自动链接架构
/// 4. 编译：cargo ndk --platform 35 --target aarch64-linux-android build --release
pub fn main() {
    // 告诉 Cargo 链接 Android 的 log 库
    println!("cargo:rustc-link-lib=log");

    // 重新编译的触发条件
    println!("cargo:rerun-if-changed=src/lib.rs");
}
