use crate::notify::{NotificationError, NotificationEvent, Notifier};
use async_trait::async_trait;
use std::path::PathBuf;
use tokio::process::Command;

pub struct WindowsNotifier {
    pub powershell: PathBuf,
}

fn escape_xml(value: &str) -> String {
    let mut result = String::with_capacity(value.len() + 32);
    for ch in value.chars() {
        match ch as u32 {
            0x26 => {
                result.push('&');
                result.push_str("amp;");
            } // &
            0x3C => {
                result.push('&');
                result.push_str("lt;");
            } // <
            0x3E => {
                result.push('&');
                result.push_str("gt;");
            } // >
            0x22 => {
                result.push('&');
                result.push_str("quot;");
            } // "
            0x27 => {
                result.push('&');
                result.push_str("apos;");
            } // '
            _ => result.push(ch),
        }
    }
    result
}

const TOAST_TEMPLATE: &str = r#"<toast>
  <visual>
    <binding template="ToastGeneric">
      <text>{title}</text>
      <text>{body}</text>
    </binding>
  </visual>
</toast>"#;

fn dash(s: &str) -> String {
    let mut result = String::with_capacity(s.len() + 1);
    result.push('-');
    result.push_str(s);
    result
}

fn ps_args() -> Vec<String> {
    vec![
        dash("NoProfile"),
        dash("NonInteractive"),
        dash("ExecutionPolicy"),
        "Bypass".to_string(),
        dash("Command"),
    ]
}

fn build_script(toast_xml: &str) -> String {
    let mut script = String::new();
    script.push_str(
        r#"
            [Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] | Out-Null
            [Windows.Data.Xml.Dom.XmlDocument, Windows.Data.Xml.Dom.XmlDocument, ContentType = WindowsRuntime] | Out-Null

            $xml = New-Object Windows.Data.Xml.Dom.XmlDocument
            $xml.LoadXml(@"
"#,
    );
    script.push_str(toast_xml);
    script.push_str(
        r#"
"@)

            $toast = [Windows.UI.Notifications.ToastNotification]::new($xml)
            $notifier = [Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier("workflow-mcp")
            $notifier.Show($toast)
"#,
    );
    script
}

#[async_trait]
impl Notifier for WindowsNotifier {
    async fn notify(&self, event: NotificationEvent) -> Result<(), NotificationError> {
        let title = escape_xml(&event.title);
        let body = escape_xml(&event.body);
        let toast_xml = TOAST_TEMPLATE
            .replace("{title}", &title)
            .replace("{body}", &body);

        let script = build_script(&toast_xml);

        let mut args = ps_args();
        args.push(script);

        let output = Command::new(&self.powershell)
            .args(args)
            .output()
            .await
            .map_err(|e| NotificationError::Unavailable(e.to_string()))?;

        if !output.status.success() {
            return Err(NotificationError::Command(
                String::from_utf8_lossy(&output.stderr).into_owned(),
            ));
        }

        Ok(())
    }
}
