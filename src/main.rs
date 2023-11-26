use std::fs;
use std::path::{Path, PathBuf};
use std::io;
use std::fs::File;
use nfd::Response;
use std::error::Error;
use std::time::SystemTime;
use chrono::{DateTime, Local};
use fltk::{app::*, group::*};
use fltk::{button::Button, frame::Frame,  window::*};
use fltk::enums::FrameType;
use fltk::prelude::*;
use std::option::Option as StdOption; 
use fltk::enums::Color;
use fltk::input::Input;
use std::io::Write;
use fltk::enums::Event;

use fltk::{prelude::*, *};
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
    last_accessed: StdOption<String>,
    last_modified: StdOption<String>,
    created: StdOption<String>,
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

fn format_system_time(st: StdOption<SystemTime>) -> StdOption<String> {
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
    /*
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
    }*/
    let app = app::App::default().with_scheme(app::Scheme::Gtk);
    let mut wind = window::Window::new(100, 100, 800, 600, "Welcome Screen");
    wind.make_resizable(true);
    wind.set_color(Color::Gray0); // Set the background color to Cyan
    //let mut frame = Frame::default().with_size(200, 100).center_of(&wind);
    let mut but = Button::new(360, 320, 65, 30, "Scan!");
    let mut but1 = Button::new(560, 250, 65, 30, "Search!");
    let mut input = Input::new(250, 250, 300, 30, ""); // Input field coordinates and size
    let placeholder_text = "Enter the directory for scanning";
    but.set_color(Color::Dark2);
    input.set_text_size(10); // Set the text size within the input field

    let mut path_to_scan= Default::default();
    ;
    input.set_value(placeholder_text); // Set the initial placeholder text
    let mut input_clone = input.clone();

    but1.set_callback(move |_| {
        let file_path;
        let result = nfd::open_pick_folder(None).unwrap_or(Response::Cancel);
    
        match result {
            Response::Okay(f) => {file_path = f.clone();},
            Response::Cancel => {file_path = String::from("No Folder is Selected");},
            _ => { file_path =  String::from("Error Selecting a Folder"); },
        }
        input_clone.set_value(&file_path);
        println!("{}", input_clone.value());

        path_to_scan = file_path;
        let mut file = match File::create("path.txt") {
            Ok(file) => file,
            Err(e) => {
                eprintln!("Error creating file: {}", e);
                return;
            }
        };
    
        // Write the text to the file
        match file.write_all(input_clone.value().as_bytes()) {
            Ok(_) => println!("Text successfully written to file!"),
            Err(e) => eprintln!("Error writing to file: {}", e),
        }
    
    });
    wind.end();
    wind.show();
    //but.set_callback(move |_| frame.set_label("Hello world"));

    app.run().unwrap();
}

