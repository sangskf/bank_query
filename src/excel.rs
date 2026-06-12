use calamine::{Data, Reader, Xlsx, open_workbook_from_rs};
use rust_xlsxwriter::*;
use std::io::Cursor;

pub fn generate_template() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut workbook = Workbook::new();
    let sheet = workbook.add_worksheet();

    let header_format = Format::new()
        .set_bold()
        .set_border(FormatBorder::Thin)
        .set_background_color(Color::Silver);

    sheet.write_string_with_format(0, 0, "联行号", &header_format)?;
    sheet.write_string_with_format(0, 1, "金融机构名称", &header_format)?;

    sheet.set_column_width(0, 20)?;
    sheet.set_column_width(1, 40)?;

    sheet.write_string(1, 0, "ICBC")?;
    sheet.write_string(1, 1, "中国工商银行")?;
    sheet.write_string(2, 0, "ABC")?;
    sheet.write_string(2, 1, "中国农业银行")?;

    Ok(workbook.save_to_buffer()?)
}

/// Try to parse bank records from a worksheet range.
fn parse_sheet(range: &calamine::Range<Data>) -> Vec<(String, String)> {
    let mut records = Vec::new();
    for row in range.rows().skip(1) {
        if row.len() < 2 {
            continue;
        }
        let code = row[0].to_string().trim().to_string();
        let name = row[1].to_string().trim().to_string();
        if !code.is_empty() && !name.is_empty() {
            records.push((code, name));
        }
    }
    records
}

pub fn parse_excel(data: &[u8]) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
    // Try .xlsx format first
    let cursor = Cursor::new(data);
    if let Ok(mut workbook) = open_workbook_from_rs::<Xlsx<_>, _>(cursor) {
        let names = workbook.sheet_names().to_vec();
        for name in &names {
            if let Ok(range) = workbook.worksheet_range(name) {
                let records = parse_sheet(&range);
                if !records.is_empty() {
                    return Ok(records);
                }
            }
        }
        return Ok(Vec::new());
    }

    Err("无法解析文件，请确保是有效的 .xlsx 格式".into())
}
