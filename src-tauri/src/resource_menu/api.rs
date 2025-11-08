use async_trait::async_trait;

pub enum HyprkubeMenuItem<C> {
    Action(HyprkubeActionMenuItem<C>),
    Separator,
    Submenu(HyprkubeActionSubMenuItem<C>),
}

#[async_trait]
pub trait MenuAction<C>: Send + Sync {
    async fn run(&self, app: &tauri::AppHandle, ctx: C);
}

pub struct HyprkubeActionMenuItem<C> {
    pub id: String,
    pub text: String,
    pub action: Box<dyn MenuAction<C>>,
}

pub struct HyprkubeActionSubMenuItem<C> {
    pub text: String,
    pub items: Vec<HyprkubeMenuItem<C>>,
}
