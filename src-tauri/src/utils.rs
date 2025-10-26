use std::fs;
use std::path::PathBuf;

/// Recursively find exe files with their folder paths
pub fn find_exe_with_folder_path(start_folder: &PathBuf) -> Option<(String, Vec<String>)> {
    let mut all_exes = Vec::new();
    
    // Collect all exe files recursively with their paths
    collect_exe_files_with_path(start_folder, start_folder, &mut all_exes);
    
    if all_exes.is_empty() {
        return None;
    }
    
    // Prioritize exe files with Chinese keywords
    let chinese_keywords = ["ch", "chs", "cn", "中文", "chinese", "简体", "繁体", "汉化"];
    
    for (exe_path, folder_names) in &all_exes {
        let file_name = exe_path.to_lowercase();
        for keyword in &chinese_keywords {
            if file_name.contains(keyword) {
                return Some((exe_path.clone(), folder_names.clone()));
            }
        }
    }
    
    // If no Chinese keyword found, return the first exe
    all_exes.into_iter().next()
}

fn collect_exe_files_with_path(
    current: &PathBuf,
    start_folder: &PathBuf,
    result: &mut Vec<(String, Vec<String>)>
) {
    if let Ok(entries) = fs::read_dir(current) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext.eq_ignore_ascii_case("exe") {
                        let exe_path = path.to_string_lossy().to_string();
                        let folder_path = build_folder_path(&path, start_folder);
                        result.push((exe_path, folder_path));
                    }
                }
            } else if path.is_dir() {
                collect_exe_files_with_path(&path, start_folder, result);
            }
        }
    }
}

fn build_folder_path(exe_path: &PathBuf, start_folder: &PathBuf) -> Vec<String> {
    let mut folder_names = Vec::new();
    
    if let Ok(relative) = exe_path.strip_prefix(start_folder) {
        for component in relative.components() {
            if let Some(name) = component.as_os_str().to_str() {
                if !name.ends_with(".exe") {
                    folder_names.push(name.to_string());
                }
            }
        }
    }
    
    folder_names
}
