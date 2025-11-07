#[cfg_attr(mobile, tauri::mobile_entry_point)]
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
};

mod ip_location;
mod ipv4_subnet;
mod ipv4_summary;
mod ipv6_subnet;
mod isp_manager;
mod nat_batch_generator;
mod nat_parser;

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&quit_i])?;

            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .show_menu_on_left_click(true)
                .icon(app.default_window_icon().unwrap().clone())
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        app.exit(0);
                    }
                    other => {
                        println!("menu item {} not handled", other);
                    }
                })
                .build(app)?;

            Ok(())
        })
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            ipv4_subnet::get_public_ip,
            ipv4_subnet::compute_subnet,
            ipv6_subnet::compute_ipv6_subnet,
            ipv4_summary::aggregate_ipv4,
            ip_location::lookup_ip_location,
            ip_location::get_database_info,
            ip_location::debug_ip_query,
            nat_parser::parse_nat_config,
            nat_batch_generator::process_excel_data,
            nat_batch_generator::convert_excel_to_entries,
            nat_batch_generator::generate_nat_commands,
            nat_batch_generator::split_port_ranges,
            nat_batch_generator::export_nat_template,
            nat_batch_generator::export_nat_commands,
            isp_manager::detect_isp_info,
            isp_manager::add_elastic_ip_mapping,
            isp_manager::bulk_add_elastic_ip_mappings,
            isp_manager::get_elastic_ip_mapping,
            isp_manager::remove_elastic_ip_mapping,
            isp_manager::get_all_elastic_mappings,
            isp_manager::get_isp_list,
            isp_manager::get_isp_data,
            isp_manager::update_isp_data,
            isp_manager::get_isp_summary,
            isp_manager::get_next_available_elastic_ip,
            isp_manager::update_isp_from_github
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
