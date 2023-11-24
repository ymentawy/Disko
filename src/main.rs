use std::fs;
use std::path::{Path, PathBuf};
use std::io;
use std::time::SystemTime;
use chrono::{DateTime, Local};

#[derive(Debug)]
struct Configurations {
    is_file: bool,
    max_depth: usize,
}

#[derive(Debug)]
struct DiskItem {
    name: String,
    is_file: bool,
    size: u64,
    last_accessed: Option<String>,
    last_modified: Option<String>,
    created: Option<String>,
    depth: usize,
    path: String,
    children: Vec<DiskItem>,
}

fn calculate_disk_usage(item: &DiskItem) -> u64 {
    if item.is_file {
        item.size
    } else {
        let children_size: u64 = item.children.iter().map(|child| calculate_disk_usage(child)).sum();
        children_size
    }
}

fn format_system_time(st: Option<SystemTime>) -> Option<String> {
    st.map(|time| {
        let datetime: DateTime<Local> = time.into();
        datetime.format("%Y-%m-%d %H:%M").to_string()
    })
}

fn scan_directory(path: &Path, depth: usize, configs: &Configurations) -> io::Result<DiskItem> {
    let metadata = fs::metadata(path)?;
    let path_str = path.to_string_lossy().into_owned();

    let mut item = DiskItem {
        name: path.file_name().unwrap().to_string_lossy().into_owned(),
        is_file: metadata.is_file(),
        size: metadata.len(),
        last_accessed: format_system_time(metadata.accessed().ok()),
        last_modified: format_system_time(metadata.modified().ok()),
        created: format_system_time(metadata.created().ok()),
        depth,
        path: path_str,
        children: Vec::new(),
    };

    if metadata.is_dir() {
        if (depth+1) < configs.max_depth {
            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let child = scan_directory(&entry.path(), depth + 1, configs)?;
                        if (configs.is_file && child.is_file) {
                            item.children.push(child);
                        }
                    }
                }
                item.size = calculate_disk_usage(&item);
            }
        }
    }

    Ok(item)
}

fn main() {
    let directory_path = Path::new("/home/youssif-abuzied/ana_disk/src");
    let configs = Configurations {
        is_file: true, // Change this to true if only files are to be included
        max_depth: 1,   // Change this depth limit as needed
    };

    match scan_directory(&directory_path, 0, &configs) {
        Ok(result) => {
            // Printing the result
            println!("{:#?}", result);
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}

