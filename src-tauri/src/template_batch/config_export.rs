use rust_xlsxwriter::Workbook;

use super::dto::GenericGeneratedConfig;

#[tauri::command]
pub fn export_template_configs(
    path: String,
    configs: Vec<GenericGeneratedConfig>,
) -> Result<(), String> {
    if configs.is_empty() {
        return Err("没有可导出的配置".into());
    }

    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    for (col, config) in configs.iter().enumerate() {
        worksheet
            .write(0u32, col as u16, &config.label)
            .map_err(|err| err.to_string())?;
        for (row_offset, line) in config.config.lines().enumerate() {
            worksheet
                .write((row_offset + 1) as u32, col as u16, line)
                .map_err(|err| err.to_string())?;
        }
    }

    workbook.save(path).map_err(|err| err.to_string())
}
