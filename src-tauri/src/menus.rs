use async_trait::async_trait;

pub enum HyprkubeMenuItem {
    Action(HyprkubeActionMenuItem),
    Separator,
    Submenu(HyprkubeActionSubMenuItem),
}

#[async_trait]
pub trait MenuAction: Send + Sync {
    async fn run(&self, app: &tauri::AppHandle, client: kube::Client) -> anyhow::Result<()>;
}

pub struct HyprkubeActionMenuItem {
    pub id: String,
    pub text: String,
    pub enabled: bool,
    pub action: Box<dyn MenuAction>,
}

pub struct HyprkubeActionSubMenuItem {
    pub text: String,
    pub items: Vec<HyprkubeMenuItem>,
}
