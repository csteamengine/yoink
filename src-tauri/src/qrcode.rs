use qrcode::QrCode;
use qrcode::render::svg;

#[tauri::command]
pub async fn generate_qr_code(content: String) -> Result<String, String> {
    let code = QrCode::new(content.as_bytes()).map_err(|e| e.to_string())?;

    let svg = code.render::<svg::Color>()
        .min_dimensions(200, 200)
        .max_dimensions(400, 400)
        .build();

    Ok(svg)
}
