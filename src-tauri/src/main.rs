// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod db;
mod models;
mod schema_data;
mod schema_settings;
mod settings_ops;
mod task_ops;

use std::collections::HashMap;

use chrono::NaiveDate;
use db::run_migrations;
use settings_ops::check_settings;
use settings_ops::get_settings;
use settings_ops::update_settings;
use task_ops::delete;
use task_ops::edit;
use task_ops::get_tasks;
use task_ops::insert_task;
use task_ops::update;

use window_shadows::set_shadow;

use tauri::Manager;

fn main() {
    #[cfg(any(windows, target_os = "windows"))]  
    check_settings();
    tauri::Builder::default()
        .setup(|app| {
            let window = app.get_window("main").unwrap();
            set_shadow(&window, true).expect("Unsupported platform!");

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            fetch_tasks,
            insert,
            save_settings,
            fetch_settings,
            update_task,
            edit_task,
            delete_task,
            get_db_url,
            get_isfirstrun,
            get_edit_hotkey
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn insert(
    title: &str,
    description: &str,
    author: &str,
    year: u16,
    month: u8,
    day: u8,
    done: bool,
    db_url: &str,
) {
    println!("Command received from frontend: insert task");
    match insert_task(
        String::from(title),
        Some(String::from(description)),
        String::from(author),
        NaiveDate::from_ymd_opt(year.into(), month.into(), day.into()),
        done,
        db_url,
    ) {
        Ok(inserted_rows) => {
            println!("Inserted {} row(s)", inserted_rows);
        }
        Err(e) => eprintln!("Error inserting user: {}", e),
    }
}

#[tauri::command]
fn fetch_tasks(year: u16, month: u8, day: u8, db_url: &str) -> Vec<String> {
    //println!("Getting tasks for YMD: {}/{}/{}", year, month, day);
    let date_filter = NaiveDate::from_ymd_opt(year.into(), month.into(), day.into());

    let mut results: Vec<String> = Vec::new();

    match get_tasks(date_filter, db_url) {

        Ok(tasks) => {
            println!("Displaying {} tasks", tasks.len());
            for task in tasks {
                println!("{:?}", task);
                results.push(task.to_string());
                //println!("testo: {}", task.get_data(2));
            }
        }
        Err(e) => eprintln!("Error reading tasks: {}", e),
    }
    let json_results = serde_json::to_string(&results).expect("Failed to serialize as JSON");
    let parsed_results: Vec<String> =
        serde_json::from_str(&json_results).expect("Failed to parse JSON");

    return parsed_results;
}

#[tauri::command]
fn save_settings(username: &str, db_url: &str, language: &str, edit_k: &str) {
    println!("HELLO");
    println!(
        "saving settings, new url: {}, new username: {}, new language: {}, new editmode hotkey: {}",
        db_url, username, language, edit_k
    );
    update_settings(
        username.to_string(),
        Some(db_url.to_string()),
        language.to_string(),
        false,
        edit_k.to_string(),
    );
}

#[tauri::command]
fn fetch_settings() -> Vec<String> {
    let mut results: Vec<String> = Vec::new();

    match get_settings() {
        Ok(settings) => {
            for setting in settings {
                //println!("{:?}", setting);
                results.push(setting.to_string());
                //println!("testo: {}", task.get_data(2));
            }
        }
        Err(e) => eprintln!("Error reading settings: {}", e),
    }
    let json_results =
        serde_json::to_string(&results).expect("Failed to serialize settings as JSON");
    let parsed_results: Vec<String> =
        serde_json::from_str(&json_results).expect("Failed to parse settings JSON");

    return parsed_results;
}

#[tauri::command]
fn update_task(id: &str, status: bool, db_url: &str) {
    let parsed_id: Result<i32, _> = id.parse();

    match parsed_id {
        Ok(parsed) => {
            update(parsed, status, db_url);
        }
        Err(_) => {
            println!("Failed to parse the string as an i32.");
        }
    }
}

#[tauri::command]
fn edit_task(id: &str, db_url: &str, title: &str, description: &str) {
    let parsed_id: Result<i32, _> = id.parse();

    match parsed_id {
        Ok(parsed) => {
            edit(parsed, title, description, db_url);
        }
        Err(_) => {
            println!("Failed to parse the string as an i32.");
        }
    }
}

#[tauri::command]
fn delete_task(id: &str, db_url: &str) {
    let parsed_id: Result<i32, _> = id.parse();

    match parsed_id {
        Ok(parsed) => {
            delete(parsed, db_url);
        }
        Err(_) => {
            println!("Failed to parse the string as an i32.");
        }
    }
}

#[tauri::command]
fn get_db_url() -> String {
    //println!("TEST ATTENZIONE");

    let mut results: Vec<String> = Vec::new();

    match get_settings() {
        Ok(settings) => {
            for setting in settings {
                results.push(setting.to_string());
            }
        }
        Err(e) => eprintln!("Error reading settings: {}", e),
    }

    let mut parsed_results: Vec<HashMap<String, String>> = Vec::new();

    for setting_str in &results {
        let key_value_pairs: Vec<&str> = setting_str.split(',').collect();
        let mut parsed_setting: HashMap<String, String> = HashMap::new();

        for pair in key_value_pairs {
            let components: Vec<&str> = pair.trim().split('=').collect();
            if components.len() == 2 {
                let key = components[0].trim().to_string();
                let value = components[1].trim().to_string();
                parsed_setting.insert(key, value);
            }
        }

        parsed_results.push(parsed_setting);
    }

    //println!("{:?}", parsed_results);

    if let Some(parsed_setting) = parsed_results.get(0) {
        if let Some(data_database_url) = parsed_setting.get("data_database_url") {
            if data_database_url != "data.db" {
                run_migrations(&data_database_url)
            }
            return data_database_url.to_string();
        } else {
            return "database url not found in settings".to_string();
        }
    } else {
        return "settings not found".to_string();
    }
}

#[tauri::command]
fn get_isfirstrun() -> bool {
    //println!("TEST ATTENZIONE");

    let mut results: Vec<String> = Vec::new();

    match get_settings() {
        Ok(settings) => {
            for setting in settings {
                results.push(setting.to_string());
            }
        }
        Err(e) => eprintln!("Error reading settings: {}", e),
    }

    let mut parsed_results: Vec<HashMap<String, String>> = Vec::new();

    for setting_str in &results {
        let key_value_pairs: Vec<&str> = setting_str.split(',').collect();
        let mut parsed_setting: HashMap<String, String> = HashMap::new();

        for pair in key_value_pairs {
            let components: Vec<&str> = pair.trim().split('=').collect();
            if components.len() == 2 {
                let key = components[0].trim().to_string();
                let value = components[1].trim().to_string();
                parsed_setting.insert(key, value);
            }
        }

        parsed_results.push(parsed_setting);
    }

    //println!("{:?}", parsed_results);
    // Iterate through each parsed setting
    for parsed_setting in &parsed_results {
        // Access the value associated with the "firstrun" key in the current setting
        if let Some(firstrun_value) = parsed_setting.get("firstrun") {
            // Parse the string value into a boolean
            match firstrun_value.parse::<bool>() {
                Ok(parsed_bool) => {
                    println!("Is first run: {}", parsed_bool);
                    return parsed_bool;
                }
                Err(_) => {
                    println!("Unable to parse the string as a boolean");
                }
            }
        } else {
            return false;
        }
    }

    return true;
}

#[tauri::command]
fn get_edit_hotkey() -> String {
    //println!("TEST ATTENZIONE");

    let mut results: Vec<String> = Vec::new();

    match get_settings() {
        Ok(settings) => {
            for setting in settings {
                results.push(setting.to_string());
            }
        }
        Err(e) => eprintln!("Error reading settings: {}", e),
    }

    let mut parsed_results: Vec<HashMap<String, String>> = Vec::new();

    for setting_str in &results {
        let key_value_pairs: Vec<&str> = setting_str.split(',').collect();
        let mut parsed_setting: HashMap<String, String> = HashMap::new();

        for pair in key_value_pairs {
            let components: Vec<&str> = pair.trim().split('=').collect();
            if components.len() == 2 {
                let key = components[0].trim().to_string();
                let value = components[1].trim().to_string();
                parsed_setting.insert(key, value);
            }
        }

        parsed_results.push(parsed_setting);
    }

    //println!("{:?}", parsed_results);

    if let Some(parsed_setting) = parsed_results.get(0) {
        if let Some(edit_hotkey) = parsed_setting.get("edit_hotkey") {
            return edit_hotkey.to_string();
        } else {
            return "edit_hotkey not found in settings".to_string();
        }
    } else {
        return "settings not found".to_string();
    }
}