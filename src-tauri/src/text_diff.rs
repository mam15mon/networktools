use serde::{Deserialize, Serialize};
use similar::TextDiff;

#[derive(Deserialize)]
pub struct TextDiffRequest {
    pub left: String,
    pub right: String,
    #[serde(default)]
    pub left_name: Option<String>,
    #[serde(default)]
    pub right_name: Option<String>,
    #[serde(default)]
    pub context: Option<usize>,
}

#[derive(Serialize)]
pub struct TextDiffSummary {
    pub left_lines: usize,
    pub right_lines: usize,
    pub insertions: usize,
    pub deletions: usize,
    pub equal: usize,
}

#[derive(Serialize)]
pub struct TextDiffResponse {
    pub patch: String,
    pub summary: TextDiffSummary,
}

#[tauri::command]
pub fn generate_text_diff(req: TextDiffRequest) -> Result<TextDiffResponse, String> {
    let context = req.context.unwrap_or(500);
    let left_name = req.left_name.unwrap_or_else(|| "文本A".to_string());
    let right_name = req.right_name.unwrap_or_else(|| "文本B".to_string());

    let diff = TextDiff::from_lines(&req.left, &req.right);

    let mut insertions = 0usize;
    let mut deletions = 0usize;
    let mut equal = 0usize;

    for op in diff.ops() {
        let old_len = op.old_range().len();
        let new_len = op.new_range().len();
        match op.tag() {
            similar::DiffTag::Insert => insertions += new_len,
            similar::DiffTag::Delete => deletions += old_len,
            similar::DiffTag::Equal => equal += old_len.max(new_len),
            similar::DiffTag::Replace => {
                insertions += new_len;
                deletions += old_len;
            }
        }
    }

    let mut patch_bytes = Vec::new();
    diff.unified_diff()
        .context_radius(context)
        .header(&left_name, &right_name)
        .to_writer(&mut patch_bytes)
        .map_err(|err| err.to_string())?;
    let patch = String::from_utf8(patch_bytes).map_err(|err| err.to_string())?;

    Ok(TextDiffResponse {
        patch,
        summary: TextDiffSummary {
            left_lines: req.left.lines().count(),
            right_lines: req.right.lines().count(),
            insertions,
            deletions,
            equal,
        },
    })
}
