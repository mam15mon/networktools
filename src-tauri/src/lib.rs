#[cfg_attr(mobile, tauri::mobile_entry_point)]

use tauri::{
	menu::{Menu, MenuItem},
	tray::TrayIconBuilder
};

mod ipv4_subnet;
mod ipv6_subnet;
mod ipv4_summary;
mod ip_location;
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
			nat_parser::parse_nat_config
		])
		.run(tauri::generate_context!())
		.expect("error while running tauri application");
}
