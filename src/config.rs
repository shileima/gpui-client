use std::env;
use std::fs;
use std::path::PathBuf;

use serde::Deserialize;

const DEFAULT_AIGC_PORT: u16 = 20318;
const DEFAULT_WEB_PORT: u16 = 38472;
const DEFAULT_AICHAT_REMOTE: &str = "https://aichat.sankuai.com/remoteEntry.js";

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub web_port: u16,
    pub web_url: String,
    pub aichat_remote_url: String,
    pub aigc_module_path: Option<PathBuf>,
    pub default_aigc_port: u16,
    pub user_mis: String,
    pub nc_token: Option<String>,
    pub workspace_root: PathBuf,
}

#[derive(Debug, Deserialize)]
struct UserConfigFile {
    #[serde(rename = "ncToken")]
    nc_token: Option<String>,
    #[serde(rename = "userMis")]
    user_mis: Option<String>,
}

impl AppConfig {
    pub fn load() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let automan_dir = home.join(".automan");
        let workspace_root = automan_dir.join("personal-workspace");
        let _ = fs::create_dir_all(&workspace_root);
        let file_config = load_user_config_file(&home);

        let web_port = env::var("GPUI_CHAT_WEB_PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(DEFAULT_WEB_PORT);

        let web_url = env::var("GPUI_CHAT_WEB_URL")
            .unwrap_or_else(|_| format!("http://127.0.0.1:{web_port}/"));

        let aichat_remote_url = env::var("GPUI_AICHAT_REMOTE_URL")
            .or_else(|_| env::var("VITE_AICHAT_HOST").map(|h| format!("{h}/remoteEntry.js")))
            .unwrap_or_else(|_| DEFAULT_AICHAT_REMOTE.to_string());

        let aigc_module_path = env::var("GPUI_AIGC_MODULE_PATH")
            .ok()
            .map(PathBuf::from)
            .or_else(|| {
                let candidate = PathBuf::from("/Users/shilei/code/waimai-qa-aie-fe/electron/modules/aigc/dist/main.js");
                if candidate.exists() {
                    Some(candidate)
                } else {
                    None
                }
            });

        let user_mis = env::var("GPUI_USER_MIS")
            .ok()
            .or_else(|| file_config.as_ref().and_then(|c| c.user_mis.clone()))
            .unwrap_or_else(|| "gpui-user".to_string());

        let nc_token = env::var("GPUI_NC_TOKEN")
            .ok()
            .or_else(|| file_config.and_then(|c| c.nc_token));

        Self {
            web_port,
            web_url,
            aichat_remote_url,
            aigc_module_path,
            default_aigc_port: DEFAULT_AIGC_PORT,
            user_mis,
            nc_token,
            workspace_root,
        }
    }
}

fn load_user_config_file(home: &PathBuf) -> Option<UserConfigFile> {
    let path = home.join(".gpui-client/config.json");
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}
