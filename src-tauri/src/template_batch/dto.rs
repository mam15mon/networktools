use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzeTeraTemplateRequest {
    pub file_path: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TeraTemplateAnalysis {
    pub variables: Vec<String>,
    pub variable_count: usize,
    pub has_loops: bool,
    pub has_conditionals: bool,
    pub loop_count: usize,
    pub conditional_count: usize,
    #[serde(default)]
    pub iterable_variables: Vec<String>,
    #[serde(default)]
    pub iterable_fields: HashMap<String, Vec<String>>,
    #[serde(default)]
    pub sample_values: HashMap<String, Vec<String>>,
    #[serde(default)]
    pub default_fallbacks: HashMap<String, String>,
    #[serde(default)]
    pub filter_usage: HashMap<String, Vec<String>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportTeraTemplateRequest {
    pub path: String,
    pub variables: Vec<String>,
    #[serde(default)]
    pub iterable_variables: Vec<String>,
    #[serde(default)]
    pub iterable_fields: HashMap<String, Vec<String>>,
    #[serde(default)]
    pub sample_values: HashMap<String, Vec<String>>,
    #[serde(default)]
    pub default_fallbacks: HashMap<String, String>,
    #[serde(default)]
    pub filter_usage: HashMap<String, Vec<String>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewTemplateExcelRequest {
    pub file_path: String,
    pub sheet_name: Option<String>,
    pub expected_variables: Vec<String>,
    #[serde(default)]
    pub iterable_variables: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateExcelPreview {
    pub sheet_names: Vec<String>,
    pub selected_sheet: String,
    pub header_row_index: usize,
    pub columns: Vec<String>,
    pub preview_rows: Vec<Vec<String>>,
    pub total_rows: usize,
    pub columns_with_data: Vec<String>,
    #[serde(default)]
    pub invalid_iterable_columns: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateTemplateConfigsRequest {
    pub template_path: String,
    pub excel_path: String,
    pub sheet_name: Option<String>,
    pub expected_variables: Vec<String>,
    pub label_field: Option<String>,
    #[serde(default)]
    pub iterable_variables: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GenericGeneratedConfig {
    pub label: String,
    pub config: String,
    pub row_index: usize,
}
