fn main() {
    // 添加自定义的 NSIS 卸载脚本
    println!("cargo:rustc-env=TAURI_NSIS_PREINSTALL=");
    println!("cargo:rustc-env=TAURI_NSIS_POSTINSTALL=");
    println!("cargo:rustc-env=TAURI_NSIS_UNINSTALL=RMDir /r /REBOOTOK $INSTDIR");
    
    tauri_build::build()
}
