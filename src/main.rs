use std::fs;
use std::path::{Path, PathBuf};
use std::io;
use std::fs::File;
use nfd::Response;
use std::error::Error;
use std::time::SystemTime;
use chrono::{DateTime, Local};
use fltk::{app::*, group::*};
use fltk::{button::Button, frame::Frame,  window::*, menu::*};
use fltk::enums::FrameType;
use fltk::{prelude::*, *};
use std::option::Option as StdOption; 
use fltk::enums::Color;
use fltk::input::Input;
use std::io::Write;
use fltk::enums::Event;
use fltk::misc::Chart;
use std::collections::HashMap;
use std::io::Read;
use fltk::enums::Shortcut;
use fltk_theme::{widget_themes, WidgetTheme, ThemeType};use fltk::{prelude::*, *};
use fltk_theme::color_themes;
use fltk_theme::ColorTheme;
use serde::Serialize;
use serde::Deserialize;
use regex::Regex;
use fltk::enums::Align;
use fltk_theme::{SchemeType, WidgetScheme};
use fltk::{
    app, dialog,
    enums::{CallbackTrigger,  Font},
    menu,
    prelude::*,
    printer, text, window,
};
use std::{
    error,
    ops::{Deref, DerefMut},
    path,
};
use std::ffi::OsStr;
use dirs::home_dir;
use fltk::tree::Tree;
use fltk::tree::TreeSelect;

#[derive(Copy, Clone)]
#[derive(PartialEq)]

pub enum Message {
   // Changed,
    New,
    Open,
    Save,
    //SaveAs,
    //Print,
    //Quit,
    //Cut,
    //Copy,
    //Paste,
    //About,
}
use std::cell::RefCell;
use std::rc::Rc;
#[derive(Debug, Serialize, Deserialize)]
#[derive(Clone)]
struct Configurations {
    is_file: bool,
    max_depth: u64,
    include_hidden_files: bool,
    min_size: u64, // Minimum size in bytes
    max_size: u64, // Maximum size in bytes
    use_regex: bool, // Indicates whether to use regex
    regex_pattern: StdOption<String>, // Holds the regex pattern if use_regex is true
}

#[derive(Debug, Serialize, Deserialize)]
#[derive(Clone)]
struct AppConfig {
    is_file: bool,
    max_depth: u64,
    include_hidden_files: bool,
    min_size: u64,
    max_size: u64,
    use_regex: bool,
    regex_pattern: String,
}

fn validate_config(config: &AppConfig) -> Result<(), String> {
    // Validate include_files and include_hidden_files
    // (Assuming these values can't be invalid, considering they're booleans)

    // Validate max_depth
    if config.max_depth <= 0 {
        return Err(String::from("max_depth should be greater than 0"));
    }

    // Validate min_size and max_size
    if config.min_size > config.max_size {
        return Err(String::from("Invalid size constraints"));
    }

    // Validate regex_pattern if use_regex is true
    if config.use_regex {
        // Check if the regex pattern is valid
        match regex::Regex::new(&config.regex_pattern) {
            Ok(_) => {}
            Err(_) => return Err(String::from("Invalid regex pattern")),
        }
    }

    // All validations passed, return Ok(())
    Ok(())
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
    depth: u64,
    path: String,
    children: Vec<DiskItem>,
}
fn convert_to_integer(value: &str) -> Result<u64, String> {
    match value.parse::<i32>() {
        Ok(num) => Ok(num.try_into().unwrap()),
        Err(_) => Err(String::from("Failed to parse string as integer")),
    }
}
fn calculate_disk_usage(item: &DiskItem) -> u64 {
    if item.is_file {
        item.size
    } else {
        let children_size: u64 = item.children.iter().map(|child| calculate_disk_usage(child)).sum();
        children_size
    }
}
fn get_files_sorted_alphabetically_recursive(item: &DiskItem) -> Vec<&DiskItem> {
    let mut files: Vec<&DiskItem> = vec![];

    // Function to collect files recursively
    fn collect_files<'a>(item: &'a DiskItem, files: &mut Vec<&'a DiskItem>) {
        if item.is_file {
            files.push(item);
        } else {
            for child in &item.children {
                collect_files(child, files);
            }
        }
    }

    collect_files(item, &mut files);

    // Sort files alphabetically by name
    files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    files
}
fn format_system_time(st: StdOption<SystemTime>) -> StdOption<String> {
    st.map(|time| {
        let datetime: DateTime<Local> = time.into();
        datetime.format("%Y-%m-%d %H:%M").to_string()
    })
}

fn scan_directory(path: &Path, depth: u64) -> io::Result<DiskItem> {
    let metadata = fs::metadata(path)?;
    let path_str = path.to_string_lossy().into_owned();

    let mut item = DiskItem {
        name: path.file_name().unwrap().to_string_lossy().into_owned(),
        is_file: metadata.is_file(),
        size: metadata.len(),
        last_accessed: format_system_time(metadata.accessed().ok()),
        last_modified: format_system_time(metadata.modified().ok()),
        created: format_system_time(metadata.created().ok()),
        depth: depth,
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
    if(configs.min_size > item.size || configs.max_size < item.size){
        res = true;
    }
    if(configs.use_regex){
        let Some(ref pattern) = configs.regex_pattern else { todo!() };
       // println!("{}", pattern);
        let re = Regex::new(&pattern).unwrap();
        let m = re.is_match(&item.name);
        if(!m){
            res = true;
        }
    }

    return !res;
}
fn read_configurations_from_json(file_path: &str) -> Result<Configurations, Box<dyn std::error::Error>> {
    let mut file = File::open(file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let configurations: Configurations = serde_json::from_str(&contents)?;

    Ok(configurations)
}
fn get_files_sorted_by_size(item: &DiskItem) -> Vec<&DiskItem> {
    let mut files: Vec<&DiskItem> = vec![];

    // Function to collect files recursively
    fn collect_files<'a>(item: &'a DiskItem, files: &mut Vec<&'a DiskItem>) {
        if item.is_file {
            files.push(item);
        } else {
            for child in &item.children {
                collect_files(child, files);
            }
        }
    }

    collect_files(item, &mut files);

    // Sort files by size in descending order
    files.sort_by(|a, b| b.size.cmp(&a.size)); // Reverse order here (b.size.cmp(&a.size))

    files
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
fn get_depth_one_items<'a>(filtered_result: &'a DiskItem) -> Vec<&'a DiskItem> {
    let mut depth_one_items = Vec::new();

    fn traverse_depth_one<'b>(item: &'b DiskItem, depth_one_items: &mut Vec<&'b DiskItem>) {
        if item.depth == 1 {
            depth_one_items.push(item);
        } else {
            for child in &item.children {
                traverse_depth_one(child, depth_one_items);
            }
        }
    }

    traverse_depth_one(filtered_result, &mut depth_one_items);
    depth_one_items
}
fn group_and_calculate_size(filtered_result: &DiskItem) -> HashMap<String, (u64, usize)> {
    let mut file_groups: HashMap<String, (u64, usize)> = HashMap::new();

    // Group items by file type or extension, calculate total size, and count number of files in each group
    fn group_items(item: &DiskItem, file_groups: &mut HashMap<String, (u64, usize)>) {
        if item.is_file {
            let file_extension = match item.name.rfind('.') {
                Some(index) => item.name[index + 1..].to_lowercase(),
                None => "no_extension".to_string(), // Files without extensions
            };

            let entry = file_groups.entry(file_extension).or_insert((0, 0));
            entry.0 += item.size;
            entry.1 += 1;
        }

        for child in &item.children {
            group_items(child, file_groups);
        }
    }

    group_items(filtered_result, &mut file_groups);
    file_groups
}

fn format_grouped_data(file_groups: &HashMap<String, (u64, usize)>) -> String {
    let mut result = String::new();

    for (file_type, (total_size, file_count)) in file_groups {
        result.push_str(&format!(
            "File type: {} - Total size: {} bytes - Number of files: {}\n",
            file_type, total_size, file_count
        ));
        result.push('\n');
    }

    result
}
fn group_by_size(filtered_result: &DiskItem) -> HashMap<String, (usize, u64)> {
    let mut size_groups: HashMap<String, (usize, u64)> = HashMap::new();

    // Group items by size categories
    fn categorize_size(item: &DiskItem, size_groups: &mut HashMap<String, (usize, u64)>) {
        let size_category = if item.size < 1024 {
            "Below 1KB".to_string()
        } else if item.size < 1024 * 1024 {
            "Between 1KB and 1MB".to_string()
        } else if item.size < 1024 * 1024 * 1024 {
            "Between 1MB and 1GB".to_string()
        } else {
            "Above 1GB".to_string()
        };

        let entry = size_groups.entry(size_category).or_insert((0, 0));
        entry.0 += 1; // Increment item count
        entry.1 += item.size; // Add item size to the total size

        for child in &item.children {
            categorize_size(child, size_groups);
        }
    }

    categorize_size(filtered_result, &mut size_groups);
    size_groups
}

fn format_grouped_size_data(size_groups: &HashMap<String, (usize, u64)>) -> String {
    let mut result = String::new();

    for (size_category, (item_count, total_size)) in size_groups {
        result.push_str(&format!(
            "Size Category: {} - Total size: {} bytes - Number of files: {}\n",
            size_category, total_size, item_count
        ));
        result.push('\n');
    }

    result
}
fn is_hidden(entry: &fs::DirEntry) -> bool {
    entry.file_name()
        .to_string_lossy()
        .starts_with(".")
}
fn build_directory_tree(tree: &mut Tree, path: &PathBuf) {
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            if let Ok(entry) = entry {
                if !is_hidden(&entry) {
                    let file_type = entry.file_type().ok();
                    let entry_path = entry.path();
                    let entry_path_str = entry_path.to_string_lossy().to_string();

                    if let Some(file_type) = file_type {
                        if file_type.is_dir() {
                            // Add directory to the tree
                            tree.add(&entry_path_str);
                            if let Some(path_str) = path.to_str() {
                                tree.close(path_str,false);
                            } else {
                                println!("Failed to close");

                            }
                            // Recursively build tree for the subdirectory
                            build_directory_tree(tree, &entry_path);
                        } else if file_type.is_file() {
                            // Add file to the tree (if needed)
                            tree.add(&entry_path_str);
                        }
                    }
                }
            }
        }
    }
}
fn main() {
   
    let app = app::App::default().with_scheme(app::Scheme::Gtk);
    let (s, r) = app::channel::<Message>();

    let theme = ColorTheme::new(color_themes::BLACK_THEME);
    theme.apply();
    let widget_theme = WidgetTheme::new(ThemeType::AquaClassic);
    widget_theme.apply();
    // let scheme = WidgetScheme::new(SchemeType::Clean);
    // scheme.apply();
    let mut wind = window::Window::new(100, 100, 800, 600, "Welcome Screen");
    //let mut frame = Frame::default().with_size(200, 100).center_of(&wind);
    let mut but = Button::new(360, 320, 65, 30, "Scan!");
    let mut but1 = Button::new(560, 250, 65, 30, "Search!");
    let mut input = Input::new(250, 250, 300, 30, ""); // Input field coordinates and size
    let placeholder_text = "Enter the directory for scanning";
    input.set_text_size(10); // Set the text size within the input field

    //let mut path_to_scan= Default::default();
    
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

        
    
    });
    let mut input_clone2 = input.clone();
    let mut wind_clone = wind.clone();
    let mut frame = Frame::new(287, 350, 200, 50, "");
    but.set_callback(move |_| {

        //path_to_scan = file_path;
        let mut file = match File::create("path.txt") {
            Ok(file) => file,
            Err(e) => {
                eprintln!("Error creating file: {}", e);
                return;
            }
        };
    
        // Write the text to the file
        match file.write_all(input_clone2.value().as_bytes()) {
            Ok(_) => println!("Text successfully written to file!"),
            Err(e) => eprintln!("Error writing to file: {}", e),
        }
        let contents = fs::read_to_string("path.txt")
        .expect("Should have been able to read the file");
        let directory_path = Path::new(&contents);
        
    
        let Ok(configs) = read_configurations_from_json("configs.json") else { todo!() };   
        let mut clone_configgs = configs.clone();

        let mut dir_path = directory_path.clone();
        match scan_directory(&directory_path, 0) {
            
                Ok(scanned_result) => {
                let mut new_wind = Window::new(0, 0, 4000, 3000, "New Window");
                let filtered_result = filter_items(&scanned_result, &configs);
                let mut filtered_result2 = filtered_result.clone();
                let mut filtered_result3 = filtered_result.clone();
                
                wind_clone.hide();
                new_wind.make_resizable(true);
                let mut menu_bar = MenuBar::new(0, 0, 4000, 100, "");
                menu_bar.add(
                    "&Configurations/View\t",
                    Shortcut::Ctrl | 'v',
                    menu::MenuFlag::Normal,
                        |_| println!("Opened file!"),
                );
                menu_bar.add(
                    "&Configurations/Edit\t",
                    Shortcut::Ctrl | 'e',
                    menu::MenuFlag::Normal,
                        |_| println!("Edited Configurations!"),
                );
                menu_bar.add(
                    "&Group/By Extension\t",
                    Shortcut::Ctrl | 't',
                    menu::MenuFlag::Normal,
                        |_| println!("Grouped By extension!"),
                );
                menu_bar.add(
                    "&Group/By Size\t",
                    Shortcut::Ctrl | 'g',
                    menu::MenuFlag::Normal,
                        |_| println!("Grouped By Size!"),
                );
                menu_bar.add(
                    "&Get files Sorted/By Size\t",
                    Shortcut::Ctrl | 'r',
                    menu::MenuFlag::Normal,
                        |_| println!("Sorted by size!"),
                );
                menu_bar.add(
                    "&Get files Sorted/By Name\t",
                    Shortcut::Ctrl | 'm',
                    menu::MenuFlag::Normal,
                        |_| println!("Sorted by name!"),
                );
                let filtered_2  = filtered_result.clone();
                let filtered_3 = filtered_result.clone();
                
                if let Some(mut item) = menu_bar.find_item("&Get files Sorted/By Size\t"){
                    item.set_callback(move |_|{
                        let mut popup = Window::new(600, 600, 800, 800, "Sort files by size");
                        let mut sorted = get_files_sorted_by_size(&filtered_2);
                        let mut file_name_frame = Frame::new(50, 50, 450, 600, "");
                        let mut file_size_frame = Frame::new(600, 50, 150, 600, "");
                        let mut name_content = String::from("File Name: \n \n");
                        let mut size_content = String::from("File Size \n \n");
                        let length = sorted.len();
                        let mut count : u64 = 0;
                        for file in &sorted {
                            //println!("File: {} - Size: {} bytes", file.name, file.size);
                            let mut temp:String = format!("{}. {} \n",count+1, file.name);
                            let mut temp2:String = format!("{}  Bytes\n", file.size);
                            name_content.push_str(&temp);
                            size_content.push_str(&temp2);
                            count = count +1;

                            if(count %20 == 0 || (count as usize) == length){
                                let mut file_write = File::create("count.txt").expect("Unable to create file");
                                file_write.write_all(&count.to_le_bytes())
                                    .expect("Unable to write to file");
                                break;
                            }
                          
                        }
                        file_name_frame.set_label(&name_content);
                        file_name_frame.set_label_size(20);
                        file_name_frame.set_align(Align::Left | Align::Inside);
                        
                        file_size_frame.set_label(&size_content);
                        file_size_frame.set_label_size(20);
                        file_size_frame.set_align(Align::Left | Align::Inside);

                        let mut back_but = Button::new(200, 700, 65, 30, "Back!");
                        let mut next_but = Button::new(535, 700, 65, 30, "Next!");
                        let mut filtered_22 = filtered_2.clone();
                        let mut file_name_frame_clone = file_name_frame.clone();
                        let mut file_size_frame_clone = file_size_frame.clone();
                        

                        let mut filtered_23 = filtered_2.clone();
                 
                        next_but.set_callback(move |_|{
                            println!("{}", count);
                            let mut file_read = File::open("count.txt").expect("Unable to open file");
                            let mut buffer = [0; 8]; // 8 bytes for a u64 value
                            file_read.read_exact(&mut buffer)
                                .expect("Unable to read from file");
                                count = u64::from_le_bytes(buffer);
                            if(count < length.try_into().unwrap()){
                                let mut sorted2 = get_files_sorted_by_size(&filtered_23);
                                let mut begin : u64 = count;
                                let mut end: u64 = std::cmp::min(length.try_into().unwrap(), count +20);
                                let mut temp_count = 0;
                                println!("{},  {}", begin, end);
                                let mut temp_name_content = String::from("File Name: \n \n");
                                let mut temp_size_content = String::from("File Size \n \n");
                                for temp_file in &sorted2{
                                    if(temp_count >= begin && temp_count < end){
                                        let mut temp:String = format!("{}. {} \n",temp_count+1, temp_file.name);
                                        let mut temp2:String = format!("{}  Bytes\n", temp_file.size);
                                        temp_name_content.push_str(&temp);
                                        temp_size_content.push_str(&temp2);
                                    }
                                    temp_count += 1;
                                }
                                file_name_frame_clone.set_label(&temp_name_content);
                                file_size_frame_clone.set_label(&temp_size_content);
                                count = end;
                                let mut file_write = File::create("count.txt").expect("Unable to create file");
                                file_write.write_all(&count.to_le_bytes())
                                    .expect("Unable to write to file");
                                                            
                            }
                        });
                        back_but.set_callback(move |_|{
                            //let mut sorted2 = sorted.clone();
                            let mut file_read = File::open("count.txt").expect("Unable to open file");
                            let mut buffer = [0; 8]; // 8 bytes for a u64 value
                            file_read.read_exact(&mut buffer)
                                .expect("Unable to read from file");
                                count = u64::from_le_bytes(buffer);

                            println!("{}", count);
                            if(count > 20){
                                let mut sorted2 = get_files_sorted_by_size(&filtered_22);
                                
                                let mut begin : u64 = 0;
                                if(count%20 != 0){
                                    begin = count - count%20-20;
                                }else{
                                    begin = count - 40;
                                }
                                let mut end: u64 = 0;
                                if(count%20 != 0){
                                    end = count - count%20;
                                }else{
                                    end = count -20;
                                }
                                let mut temp_count = 0;
                                let mut temp_name_content = String::from("File Name: \n \n");
                                let mut temp_size_content = String::from("File Size \n \n");
                                for temp_file in &sorted2{
                                    if(temp_count >= begin && temp_count < end){
                                        let mut temp:String = format!("{}. {} \n",temp_count+1, temp_file.name);
                                        let mut temp2:String = format!("{}  Bytes\n", temp_file.size);
                                        temp_name_content.push_str(&temp);
                                        temp_size_content.push_str(&temp2);
                                        count -=1;

                                    }
                                    temp_count += 1;
                                }
                                file_name_frame.set_label(&temp_name_content);
                                file_size_frame.set_label(&temp_size_content);
                                count = end;
                                let mut file_write = File::create("count.txt").expect("Unable to create file");
                                file_write.write_all(&count.to_le_bytes())
                                    .expect("Unable to write to file");
                            }
                        });
                        popup.show();
                        popup.end();
                        

                    }
                    );

                } 
                if let Some(mut item) = menu_bar.find_item("&Get files Sorted/By Name\t"){
                    item.set_callback(move |_|{
                        let mut popup = Window::new(600, 600, 800, 800, "Sort files by name");
                        let mut sorted = get_files_sorted_alphabetically_recursive(&filtered_3);
                        let mut file_name_frame = Frame::new(50, 50, 450, 600, "");
                        let mut file_size_frame = Frame::new(600, 50, 150, 600, "");
                        let mut name_content = String::from("File Name: \n \n");
                        let mut size_content = String::from("File Size \n \n");
                        let length = sorted.len();
                        let mut count : u64 = 0;
                        for file in &sorted {
                            //println!("File: {} - Size: {} bytes", file.name, file.size);
                            let mut temp:String = format!("{}. {} \n",count+1, file.name);
                            let mut temp2:String = format!("{}  Bytes\n", file.size);
                            name_content.push_str(&temp);
                            size_content.push_str(&temp2);
                            count = count +1;

                            if(count %20 == 0 || (count as usize) == length){
                                let mut file_write = File::create("count.txt").expect("Unable to create file");
                                file_write.write_all(&count.to_le_bytes())
                                    .expect("Unable to write to file");
                                break;
                            }
                          
                        }
                        file_name_frame.set_label(&name_content);
                        file_name_frame.set_label_size(20);
                        file_name_frame.set_align(Align::Left | Align::Inside);
                        
                        file_size_frame.set_label(&size_content);
                        file_size_frame.set_label_size(20);
                        file_size_frame.set_align(Align::Left | Align::Inside);

                        let mut back_but = Button::new(200, 700, 65, 30, "Back!");
                        let mut next_but = Button::new(535, 700, 65, 30, "Next!");
                        let mut filtered_22 = filtered_3.clone();
                        let mut file_name_frame_clone = file_name_frame.clone();
                        let mut file_size_frame_clone = file_size_frame.clone();
                        

                        let mut filtered_23 = filtered_3.clone();
                 
                        next_but.set_callback(move |_|{
                            println!("{}", count);
                            let mut file_read = File::open("count.txt").expect("Unable to open file");
                            let mut buffer = [0; 8]; // 8 bytes for a u64 value
                            file_read.read_exact(&mut buffer)
                                .expect("Unable to read from file");
                                count = u64::from_le_bytes(buffer);
                            if(count < length.try_into().unwrap()){
                                let mut sorted2 = get_files_sorted_alphabetically_recursive(&filtered_23);
                                let mut begin : u64 = count;
                                let mut end: u64 = std::cmp::min(length.try_into().unwrap(), count +20);
                                let mut temp_count = 0;
                                println!("{},  {}", begin, end);
                                let mut temp_name_content = String::from("File Name: \n \n");
                                let mut temp_size_content = String::from("File Size \n \n");
                                for temp_file in &sorted2{
                                    if(temp_count >= begin && temp_count < end){
                                        let mut temp:String = format!("{}. {} \n",temp_count+1, temp_file.name);
                                        let mut temp2:String = format!("{}  Bytes\n", temp_file.size);
                                        temp_name_content.push_str(&temp);
                                        temp_size_content.push_str(&temp2);
                                    }
                                    temp_count += 1;
                                }
                                file_name_frame_clone.set_label(&temp_name_content);
                                file_size_frame_clone.set_label(&temp_size_content);
                                count = end;
                                let mut file_write = File::create("count.txt").expect("Unable to create file");
                                file_write.write_all(&count.to_le_bytes())
                                    .expect("Unable to write to file");
                                                            
                            }
                        });
                        back_but.set_callback(move |_|{
                            //let mut sorted2 = sorted.clone();
                            let mut file_read = File::open("count.txt").expect("Unable to open file");
                            let mut buffer = [0; 8]; // 8 bytes for a u64 value
                            file_read.read_exact(&mut buffer)
                                .expect("Unable to read from file");
                                count = u64::from_le_bytes(buffer);

                            println!("{}", count);
                            if(count > 20){
                                let mut sorted2 = get_files_sorted_alphabetically_recursive(&filtered_22);
                                
                                let mut begin : u64 = 0;
                                if(count%20 != 0){
                                    begin = count - count%20-20;
                                }else{
                                    begin = count - 40;
                                }
                                let mut end: u64 = 0;
                                if(count%20 != 0){
                                    end = count - count%20;
                                }else{
                                    end = count -20;
                                }
                                let mut temp_count = 0;
                                let mut temp_name_content = String::from("File Name: \n \n");
                                let mut temp_size_content = String::from("File Size \n \n");
                                for temp_file in &sorted2{
                                    if(temp_count >= begin && temp_count < end){
                                        let mut temp:String = format!("{}. {} \n",temp_count+1, temp_file.name);
                                        let mut temp2:String = format!("{}  Bytes\n", temp_file.size);
                                        temp_name_content.push_str(&temp);
                                        temp_size_content.push_str(&temp2);
                                        count -=1;

                                    }
                                    temp_count += 1;
                                }
                                file_name_frame.set_label(&temp_name_content);
                                file_size_frame.set_label(&temp_size_content);
                                count = end;
                                let mut file_write = File::create("count.txt").expect("Unable to create file");
                                file_write.write_all(&count.to_le_bytes())
                                    .expect("Unable to write to file");
                            }
                        });
                        popup.show();
                        popup.end();
                        

                    }
                    );

                }                
                if let Some(mut item) = menu_bar.find_item("&Group/By Size\t"){
                    item.set_callback(move |_| {
                        let mut popup = Window::new(600, 600, 1500, 1300, "Group files by size");

                        let extension_groups = group_by_size(&filtered_result3);
                        let formatted_data = format_grouped_size_data(&extension_groups);  
                        let mut groups_frame = Frame::new(70, 20, 500, 200, "");
                        groups_frame.set_label(&formatted_data);
                        groups_frame.set_label_size(14);

                        let mut size_chart = Chart::new(20, 300, 700, 500, "");    
                        size_chart.set_type(misc::ChartType::Pie);
                        size_chart.set_bounds(0.0, 100.0);
                        size_chart.set_text_size(10);

                        let mut count_chart = Chart::new(800, 300, 700, 500, "");    
                        count_chart.set_type(misc::ChartType::Pie);
                        count_chart.set_bounds(0.0, 100.0);
                        count_chart.set_text_size(10);

                        let colors = [
                            enums::Color::Red,
                            enums::Color::Blue,
                            enums::Color::Green,
                            enums::Color::Magenta,
                            enums::Color::Cyan,
                            enums::Color::Yellow,
                            enums::Color::DarkRed,
                            // Add more colors as needed
                        ];
                        let mut color_cycle1 = colors.iter().cycle();
                        let mut color_cycle2 = colors.iter().cycle();

                        for  (file_type, (file_count, total_size))in extension_groups{
                            let color = color_cycle1.next().unwrap_or(&enums::Color::Black);
                            size_chart.add(total_size as f64, &file_type,*color);
                            let color = color_cycle2.next().unwrap_or(&enums::Color::Black);
                            count_chart.add(file_count as f64, &file_type,*color);
                        }

                        let mut size_frame = Frame::new(300, 850, 40, 30, "Size Pie chart");
                        let mut count_frame = Frame::new(1000, 850, 40, 30, "Count Pie chart");

                    
                        popup.show();
                        popup.end();
                    });
                }
                if let Some(mut item) = menu_bar.find_item("&Group/By Extension\t"){
                    item.set_callback(move |_| {
                        let mut popup = Window::new(600, 600, 1400, 1300, "Group files by extension");

                        let extension_groups = group_and_calculate_size(&filtered_result2);
                        let formatted_data = format_grouped_data(&extension_groups);  
                        let mut groups_frame = Frame::new(70, 20, 300, 200, "");
                        groups_frame.set_label(&formatted_data);
                        groups_frame.set_label_size(14);

                        let mut size_chart = Chart::new(20, 300, 600, 500, "");    
                        size_chart.set_type(misc::ChartType::Pie);
                        size_chart.set_bounds(0.0, 100.0);
                        size_chart.set_text_size(14);

                        let mut count_chart = Chart::new(700, 300, 600, 500, "");    
                        count_chart.set_type(misc::ChartType::Pie);
                        count_chart.set_bounds(0.0, 100.0);
                        count_chart.set_text_size(14);

                        let colors = [
                            enums::Color::Red,
                            enums::Color::Blue,
                            enums::Color::Green,
                            enums::Color::Magenta,
                            enums::Color::Cyan,
                            enums::Color::Yellow,
                            enums::Color::DarkRed,
                            // Add more colors as needed
                        ];
                        let mut color_cycle1 = colors.iter().cycle();
                        let mut color_cycle2 = colors.iter().cycle();

                        for  (file_type, (total_size, file_count))in extension_groups{
                            let color = color_cycle1.next().unwrap_or(&enums::Color::Black);
                            size_chart.add(total_size as f64, &file_type,*color);
                            let color = color_cycle2.next().unwrap_or(&enums::Color::Black);
                            count_chart.add(file_count as f64, &file_type,*color);
                        }

                        let mut size_frame = Frame::new(300, 850, 40, 30, "Size Pie chart");
                        let mut count_frame = Frame::new(1000, 850, 40, 30, "Count Pie chart");

                    
                        popup.show();
                        popup.end();
                    });
                }
                if let Some(mut item) = menu_bar.find_item("&Configurations/Edit\t"){
                    item.set_callback(move |_|{
                        let mut popup = Window::new(600, 600, 400, 400, "Edit Configurations");
                        // let filesOnlylabel = format!("Include Files: ");
                        // let mut filesOnly = Frame::new(70, 10, 70, 30, "");
                        // filesOnly.set_label(&filesOnlylabel);
                        // filesOnly.set_label_size(18);

                        let mut include_files_choice = menu::Choice::new(150, 10, 100, 30, "Include Files: ");
                        include_files_choice.add_choice(" Yes | No");
                        include_files_choice.set_value(0);
                        include_files_choice.set_color(enums::Color::White);

                        let mut include_hidden_choice = menu::Choice::new(150, 50, 100, 30, "Include Hidden Files: ");
                        include_hidden_choice.add_choice(" Yes | No");
                        include_hidden_choice.set_value(0);
                        include_hidden_choice.set_color(enums::Color::White);

                        let mut depth_input = Input::new(150, 90, 100, 30, "");
                        let mut depth_label = Frame::new(45, 90, 70, 30, "Max Scanning Depth: ");
                        depth_label.set_label_size(14);
                        depth_input.set_value("3");
                        

                        let mut minsize_input = Input::new(150, 130, 100, 30, "");
                        let mut minsize_label = Frame::new(45, 130, 70, 30, "Min File Size: ");
                        minsize_label.set_label_size(14);
                        minsize_input.set_value("500");                       
                        let mut min_size_unit = menu::Choice::new(300, 130, 80, 30, "Unit: ");
                        min_size_unit.add_choice(" Bytes | KBs | MBs | GBs ");
                        min_size_unit.set_value(0);
                        min_size_unit.set_color(enums::Color::White);

                        let mut maxsize_input = Input::new(150, 170, 100, 30, "");
                        let mut maxsize_label = Frame::new(45, 170, 70, 30, "Max File Size: ");
                        maxsize_label.set_label_size(14);
                        maxsize_input.set_value("500");                       
                        let mut max_size_unit = menu::Choice::new(300, 170, 80, 30, "Unit: ");
                        max_size_unit.add_choice(" Bytes | KBs | MBs | GBs ");
                        max_size_unit.set_value(0);
                        max_size_unit.set_color(enums::Color::White);
                        
                        let mut regex_choice = menu::Choice::new(150, 210, 100, 30, "Use Regex: ");
                        regex_choice.add_choice(" Yes | No");
                        regex_choice.set_value(0);
                        regex_choice.set_color(enums::Color::White);

                        let mut regex_input = Input::new(150, 250, 100, 30, "");
                        let mut regex_label = Frame::new(45, 250, 70, 30, "Regex Pattern: ");
                        regex_label.set_label_size(14);
                        regex_input.set_value(".rs$");
                        
                        let mut Edit_button = Button::new(150, 290, 70, 30, "Edit!");
                        let mut error_label = Frame::new(10, 330, 380, 30, "");
                        let mut popup3 = popup.clone();
                        Edit_button.set_callback(move |_|{
                            let mut err :bool = false;
                        //    let mut val :String = "";
                            let mut depth:u64 = 0;
                            match convert_to_integer(&depth_input.value()) {
                                Ok(result) => depth = result,
                                Err(err) => error_label.set_label(&"Error! Please Check the values you enetered."),
                            }

                            let mut mini_size:u64 = 0;
                            match convert_to_integer(&minsize_input.value()) {
                                Ok(result) => mini_size = result,
                                Err(err) => error_label.set_label(&"Error! Please Check the values you enetered."),
                            }

                            let mut maxi_size:u64 = 0;
                            match convert_to_integer(&maxsize_input.value()) {
                                Ok(result) => maxi_size = result,
                                Err(err) => error_label.set_label(&"Error! Please Check the values you enetered."),
                            }

                            let include_files_in : bool = (include_files_choice.value() == 0);
                            let include_hidden_files_in : bool = (include_hidden_choice.value() == 0);
                            let use_regex_in:bool = (regex_choice.value() == 0);
                            let regex_pattern_in : String = regex_input.value();

                            let base:u64 = 1024;
                            maxi_size = maxi_size * (base.pow(max_size_unit.value().try_into().unwrap()));
                            mini_size = mini_size*(base.pow(min_size_unit.value().try_into().unwrap()));
                            let temp_config = AppConfig {
                                is_file: include_files_in,
                                include_hidden_files: include_hidden_files_in,
                                max_depth: depth,
                                min_size: mini_size,
                                max_size: maxi_size,
                                use_regex: use_regex_in,
                                regex_pattern: regex_pattern_in,
                            };

                            match validate_config(&temp_config) {
                                Ok(_) => {
                                    let mut popup2 = popup.clone();

                                    let serialized = serde_json::to_string(&temp_config).expect("Serialization failed");

                                    // Write the JSON string to a file
                                    let mut json_file = File::create("configs.json").expect("File creation failed");
                                    json_file.write_all(serialized.as_bytes()).expect("Write failed");

                                    popup2.hide();
                                    
                                },
                                Err(err) => error_label.set_label(&"Error! Please Check the values you enetered."),
                            }
                        

                        });


                        popup3.show();
                        popup3.end();

                    });
                }
                //let mut configs = configs.clone(); // Cloning the data
                if let Some(mut item) = menu_bar.find_item("&Configurations/View\t") {
                    item.set_callback(move |_|{
                        let Ok(configs) = read_configurations_from_json("configs.json") else { todo!() };   

                        let mut popup = Window::new(600, 600, 400, 400, "View Configurations");
                        let filesOnlylabel = format!("Include Files:   {}", configs.is_file);
                        let mut filesOnly = Frame::new(70, 10, 70, 30, "");
                        filesOnly.set_label(&filesOnlylabel);
                        filesOnly.set_label_size(18);

                        let includeHiddenlabel = format!("Include Hidden Files:   {}", configs.include_hidden_files);
                        let mut includeHidden = Frame::new(70, 50, 140, 30, "");
                        includeHidden.set_label(&includeHiddenlabel);
                        includeHidden.set_label_size(18);

                        let maxDepthLabel = format!("Max File/Folder Depth:   {}", configs.max_depth);
                        let mut maxDepth = Frame::new(70, 90, 135, 30, "");
                        maxDepth.set_label(&maxDepthLabel);
                        maxDepth.set_label_size(18);

                        let minSizelabel = format!("Minimum Size:   {} Bytes", configs.min_size);
                        let mut minSize = Frame::new(70, 130, 140, 30, "");
                        minSize.set_label(&minSizelabel);
                        minSize.set_label_size(18);
                        
                        let maxSizelabel = format!("Maximum Size:   {} Bytes", configs.max_size);
                        let mut maxSize = Frame::new(70, 170, 170, 30, "");
                        maxSize.set_label(&maxSizelabel);
                        maxSize.set_label_size(18);

                        let useRegexabel = format!("Use Regex:   {}", configs.use_regex);
                        let mut useregex = Frame::new(70, 210, 70, 30, "");
                        useregex.set_label(&useRegexabel);
                        useregex.set_label_size(18);
                        
                        let Some(ref rpatter) = clone_configgs.regex_pattern else { todo!() };

                        let regexlabel = format!("Regex Pattern:   {}", rpatter);
                        let mut regexf = Frame::new(70, 250, 90, 30, "");
                        regexf.set_label(&regexlabel);
                        regexf.set_label_size(18);

                        popup.show();
                        popup.end();
                    });
                }
                let mut chart = Chart::new(2000, 400, 2000, 2000, "");    
                chart.set_type(misc::ChartType::Pie);
                chart.set_bounds(0.0, 100.0);
                chart.set_text_size(14);
                let mut chart_colne = chart.clone();
                let mut choice = menu::Choice::new(2800, 200, 400, 150, "Chart type");
                choice.add_choice(" Pie | SpecialPie");
                choice.set_value(0);
                choice.set_color(enums::Color::White);
                
            
                choice.set_callback(move |c| {
                    chart_colne.set_type(misc::ChartType::from_i32(c.value()+5));
                    chart_colne.redraw();
                });
                // println!("{:?}",extension_groups);
                let depth_one_items = get_depth_one_items(&filtered_result);
                let mut size_temp = filtered_result.size;
                let mut size_unit = "Bytes";
                if(size_temp >= (1024*1024*1024)){
                    size_temp = size_temp / (1024*1024*1024);
                    size_unit = "GBs";
                }else if (size_temp >= (1024*1024)){
                    size_temp = size_temp / (1024*1024);
                    size_unit = "MBs";
                }else if(size_temp >= 1024){
                    size_temp = size_temp / (1024);
                    size_unit = "KBs";
                }
                 let Sizelabel = format!("Directory size =  {} {}", size_temp, size_unit);
                 let mut size_frame = Frame::new(3000, 2500, 200, 30, "");
                 size_frame.set_label(&Sizelabel);
                 size_frame.set_label_size(18);
                
                // Display or work with the items at depth 1
                let colors = [
                    enums::Color::Red,
                    enums::Color::Blue,
                    enums::Color::Green,
                    enums::Color::Magenta,
                    enums::Color::Cyan,
                    enums::Color::Yellow,
                    enums::Color::DarkRed,
                    // Add more colors as needed
                ];
                let mut tree = Tree::new(100, 400, 1800, 2000, "");
                tree.set_select_mode(TreeSelect::Multi);
                let mut color_cycle = colors.iter().cycle();
                for item in depth_one_items {
                    println!("{:#?}", item);
                    let color = color_cycle.next().unwrap_or(&enums::Color::Black);
                    chart.add(item.size as f64, &item.name,*color)
                }
                let path_buf: PathBuf = dir_path.to_path_buf();
                if let Some(home_dir) = home_dir() {
                    build_directory_tree(&mut tree, &path_buf);
                } else {
                    eprintln!("Unable to determine the user's home directory.");
                }
                tree.callback_item();
                new_wind.show();
                new_wind.end();
             
            }
            Err(e) => {eprintln!("Error: {}", e);
            frame.set_label("Directory Not found");},
    }
      
       
    });
    wind.end();
    wind.show();
    //but.set_callback(move |_| frame.set_label("Hello world"));

    app.run().unwrap();
}

