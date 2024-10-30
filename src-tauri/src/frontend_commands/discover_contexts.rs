use kube::config::Kubeconfig;
use tauri::Manager;

use crate::frontend_types::BackendError;

pub type KubeContextSource = (String, String);

#[tauri::command]
pub async fn discover_contexts(
    app_handle: tauri::AppHandle,
) -> Result<Vec<KubeContextSource>, BackendError> {
    let home_dir = app_handle.path().home_dir().unwrap();
    let config_dir = app_handle.path().config_dir().unwrap();

    let mut contexts: Vec<KubeContextSource> = Vec::new();

    let path_default_kubeconfig = {
        let mut pathbuf = home_dir.clone();
        pathbuf.push(".kube");
        pathbuf.push("config");

        pathbuf
    };

    let kubeconfig = Kubeconfig::read_from(&path_default_kubeconfig).unwrap();
    for context in &kubeconfig.contexts {
        contexts.push((
            path_default_kubeconfig.to_str().unwrap().to_owned(),
            context.name.to_owned(),
        ));
    }

    let path_openlens_kubeconfigs = {
        let mut pathbuf = config_dir.clone();
        pathbuf.push("OpenLens");
        pathbuf.push("kubeconfigs");

        pathbuf
    };

    let mut openlens_kubeconfigs = tokio::fs::read_dir(path_openlens_kubeconfigs)
        .await
        .unwrap();

    while let Some(file) = openlens_kubeconfigs.next_entry().await.unwrap() {
        let kubeconfig = Kubeconfig::read_from(file.path()).unwrap();
        for context in &kubeconfig.contexts {
            contexts.push((
                file.path().to_str().unwrap().to_owned(),
                context.name.to_owned(),
            ));
        }
    }

    Ok(contexts)
}
