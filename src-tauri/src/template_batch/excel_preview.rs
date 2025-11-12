use calamine::{open_workbook_auto, Data, Range, Reader};

use super::dto::{PreviewTemplateExcelRequest, TemplateExcelPreview};
use super::excel_utils::{
    data_has_value, data_to_value, data_type_to_string, find_column_index, find_header_row,
    iterable_value_is_valid, row_is_empty, validate_columns,
};

const MAX_PREVIEW_ROWS: usize = 100;

#[tauri::command]
pub fn preview_template_excel(
    request: PreviewTemplateExcelRequest,
) -> Result<TemplateExcelPreview, String> {
    let mut workbook = open_workbook_auto(&request.file_path)
        .map_err(|err| format!("无法打开 Excel 文件: {err}"))?;
    let sheet_names = workbook.sheet_names().to_vec();
    if sheet_names.is_empty() {
        return Err("Excel 文件中没有工作表".into());
    }

    let selected_sheet = request
        .sheet_name
        .filter(|name| sheet_names.contains(name))
        .unwrap_or_else(|| sheet_names[0].clone());

    let range = workbook
        .worksheet_range(&selected_sheet)
        .map_err(|err| format!("读取工作表失败: {err}"))?;

    let (header_row_index, columns) =
        find_header_row(&range).ok_or_else(|| "无法定位表头行".to_string())?;

    validate_columns(&columns, &request.expected_variables)?;

    let column_count = columns.len();
    let (preview_rows, total_rows, column_non_empty_counts) =
        collect_preview_rows(&range, header_row_index, column_count);
    let columns_with_data = columns
        .iter()
        .enumerate()
        .filter_map(|(idx, name)| {
            if column_non_empty_counts
                .get(idx)
                .copied()
                .unwrap_or_default()
                > 0
            {
                Some(name.clone())
            } else {
                None
            }
        })
        .collect();

    let invalid_iterable_columns = request
        .iterable_variables
        .iter()
        .filter_map(|variable| {
            let idx = find_column_index(&columns, variable)?;
            match assess_iterable_column(&range, header_row_index, idx) {
                IterableColumnState::Invalid => Some(variable.clone()),
                _ => None,
            }
        })
        .collect();

    Ok(TemplateExcelPreview {
        sheet_names,
        selected_sheet,
        header_row_index,
        columns,
        preview_rows,
        total_rows,
        columns_with_data,
        invalid_iterable_columns,
    })
}

fn collect_preview_rows(
    range: &Range<Data>,
    header_row_index: usize,
    column_count: usize,
) -> (Vec<Vec<String>>, usize, Vec<usize>) {
    let mut preview_rows = Vec::new();
    let mut total_rows = 0usize;
    let mut column_non_empty_counts = vec![0usize; column_count];

    for (idx, row) in range.rows().enumerate() {
        if idx <= header_row_index {
            continue;
        }
        if row_is_empty(row) {
            continue;
        }
        total_rows += 1;

        if preview_rows.len() < MAX_PREVIEW_ROWS {
            let mut formatted_row = Vec::with_capacity(column_count);
            for col_idx in 0..column_count {
                let value = row
                    .get(col_idx)
                    .map(data_type_to_string)
                    .unwrap_or_default();
                if !value.is_empty() {
                    if let Some(count) = column_non_empty_counts.get_mut(col_idx) {
                        *count += 1;
                    }
                }
                formatted_row.push(value);
            }
            preview_rows.push(formatted_row);
        } else {
            for col_idx in 0..column_count {
                if let Some(cell) = row.get(col_idx) {
                    if data_has_value(cell) {
                        if let Some(count) = column_non_empty_counts.get_mut(col_idx) {
                            *count += 1;
                        }
                    }
                }
            }
        }
    }

    (preview_rows, total_rows, column_non_empty_counts)
}

enum IterableColumnState {
    Empty,
    Valid,
    Invalid,
}

fn assess_iterable_column(
    range: &Range<Data>,
    header_row_index: usize,
    column_index: usize,
) -> IterableColumnState {
    let mut seen_valid = false;
    for (idx, row) in range.rows().enumerate() {
        if idx <= header_row_index {
            continue;
        }
        if let Some(cell) = row.get(column_index) {
            if !data_has_value(cell) {
                continue;
            }
            let value = data_to_value(cell);
            if iterable_value_is_valid(&value) {
                seen_valid = true;
                continue;
            } else {
                return IterableColumnState::Invalid;
            }
        }
    }
    if seen_valid {
        IterableColumnState::Valid
    } else {
        IterableColumnState::Empty
    }
}
