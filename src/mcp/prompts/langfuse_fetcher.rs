//! ตัวดึง Prompt จาก Langfuse API พร้อม fallback สำรอง
//!
//! พยายามดึงข้อความ prompt จาก Langfuse Prompt Management API
//! หากเกิดข้อผิดพลาดหรือไม่มีการเชื่อมต่อ จะใช้ข้อความ fallback ที่กำหนดเองแทน
//! ตัวแปรแบบ {{variable_name}} ใน template จะถูกแทนที่ด้วยค่าจาก vars

use std::collections::HashMap;

/// โครงสร้าง JSON ที่ส่งกลับจาก Langfuse Prompt API
#[derive(serde::Deserialize)]
struct LangfusePromptResponse {
    #[serde(rename = "prompt")]
    prompt: Option<String>,
    #[serde(rename = "name")]
    _name: String,
}

/// ดึง prompt จาก Langfuse พร้อม substitution ตัวแปรและ fallback
///
/// # Arguments
/// - `prompt_name`: ชื่อ prompt (เช่น "flowz_workflow_compose")
/// - `vars`: แผนที่ตัวแปรสำหรับ substitution {{var}}
/// - `fallback`: ข้อความสำรองเมื่อดึงจาก Langfuse ไม่สำเร็จ
///
/// # Returns
/// ข้อความ prompt ที่ผ่านการ substitution แล้ว
pub async fn fetch_prompt_with_fallback(
    prompt_name: &str,
    vars: &HashMap<String, String>,
    fallback: &str,
) -> String {
    let template = fetch_prompt_from_langfuse(prompt_name)
        .await
        .unwrap_or_else(|| fallback.to_string());
    substitute_variables(&template, vars)
}

/// พยายมดึง prompt template จาก Langfuse API
/// คืนค่า None ถ้ามีข้อผิดพลาดใดๆ หรือไม่มีการตั้งคีย์
async fn fetch_prompt_from_langfuse(prompt_name: &str) -> Option<String> {
    let client = crate::logging::langfuse::get_client()?;
    let base_url = client.base_url().trim_end_matches('/');
    let url = format!("{}/api/public/v1/prompts/{}", base_url, prompt_name);

    let resp = client
        .http_client()
        .get(&url)
        .header("Authorization", format!("Bearer {}", client.public_key()))
        .header("Content-Type", "application/json")
        .send()
        .await
        .ok()?
        .json::<LangfusePromptResponse>()
        .await
        .ok()?;

    resp.prompt
}

/// แทนที่ {{variable_name}} ใน template ด้วยค่าจาก vars
fn substitute_variables(template: &str, vars: &HashMap<String, String>) -> String {
    let mut result = template.to_string();
    for (key, value) in vars {
        let placeholder = format!("{{{{{}}}}}", key);
        result = result.replace(&placeholder, value);
    }
    result
}
