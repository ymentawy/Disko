use std::fs;
use std::path::{Path, PathBuf};
use std::io;
use std::time::SystemTime;
use chrono::{DateTime, Local};

#[derive(Debug)]
struct Configurations {
    is_file: bool,
    max_depth: usize,
    include_hidden_files: bool,
}

#[derive(Debug)]
#[derive(Clone)]
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

fn scan_directory(path: &Path, depth: usize) -> io::Result<DiskItem> {
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
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let child = scan_directory(&entry.path(), depth + 1)?;
                    item.children.push(child);
                }
            }
            item.size = calculate_disk_usage(&item);
        }
    }

    Ok(item)
}
fn check_file(item: &DiskItem, configs: &Configurations) -> bool {
    let mut res: bool = false;
    let mut count:u64 = 0;
    if(!configs.is_file){
        res = true;
    }
    if(!configs.include_hidden_files){
        for c in item.name.chars() { 
            if c == '.' && count == 0{
                res = true;
                count = count +1;
            }
        }
    }
    return !res;
}

fn filter_items(item: &DiskItem, configs: &Configurations) -> DiskItem {
    let mut filtered_item = DiskItem {
        name: item.name.clone(),
        is_file: item.is_file,
        size: item.size,
        last_accessed: item.last_accessed.clone(),
        last_modified: item.last_modified.clone(),
        created: item.created.clone(),
        depth: item.depth,
        path: item.path.clone(),
        children: Vec::new(),
    };
 
    if(!filtered_item.is_file){
        if(configs.max_depth > filtered_item.depth){
            // filtered_item.children = item.children
            // .iter()
            // .map(|child| filter_items(child, configs))
            // .collect();
            for child in item.children.iter(){
                if(!child.is_file){
                    filtered_item.children.push(filter_items(child, configs));
                }else{
                    if(check_file(child, configs)){
                        filtered_item.children.push(child.clone());
                    }
                }


            }
        }
    }




    filtered_item
}

fn main() {
    let directory_path = Path::new("/home/youssif-abuzied/Desktop");
    let configs = Configurations {
        is_file: true, // Set to false to display only folders, true for both files and folders
        max_depth: 1,
        include_hidden_files : true,   // Adjust depth as needed
    };

    match scan_directory(&directory_path, 0) {
        Ok(scanned_result) => {
            let filtered_result = filter_items(&scanned_result, &configs);
            println!("{:#?}", filtered_result);
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}

