extern crate winres;
use std::env;
use std::path::PathBuf;

fn main() {
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();

        // 获取项目根目录的绝对路径
        let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("无法获取 CARGO_MANIFEST_DIR");
        let icon_path = PathBuf::from(&manifest_dir).join("assets").join("icon.ico");

        // 将路径转换为字符串，并使用 Windows 风格的路径分隔符
        let icon_path_str = icon_path.to_str().expect("路径转换失败").replace("/", "\\");

        println!("cargo:warning=使用图标路径: {}", icon_path_str);

        // 设置程序图标（使用完整路径）
        res.set_icon(&icon_path_str);

        // 设置版本信息
        res.set("FileDescription", "UTermux");
        res.set("ProductName", "UTermux");
        res.set("FileVersion", "1.0.0");
        res.set("LegalCopyright", "Copyright © 2024");

        // 设置语言
        res.set_language(0x0004); // 中文简体

        res.compile().expect("无法编译 Windows 资源文件");
    }
}
