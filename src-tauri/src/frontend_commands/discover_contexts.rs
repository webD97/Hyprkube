use kube::config::Kubeconfig;
use serde::{Deserialize, Serialize};
use tauri::Manager;
use tracing::warn;

use crate::frontend_types::BackendError;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct KubeContextSource {
    pub provider: String,
    pub source: String,
    pub context: String,
}

impl std::fmt::Display for KubeContextSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}://{}#{}", self.provider, self.source, self.context)
    }
}

#[tauri::command]
#[tracing::instrument(skip_all, fields(request_id = tracing::field::Empty))]
pub async fn discover_contexts(
    app_handle: tauri::AppHandle,
) -> Result<Vec<KubeContextSource>, BackendError> {
    crate::internal::tracing::set_span_request_id();

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
        contexts.push(KubeContextSource {
            provider: "file".to_owned(),
            source: path_default_kubeconfig.to_str().unwrap().to_owned(),
            context: context.name.to_owned(),
        });
    }

    for lens_compat_dir in ["OpenLens", "Lens"].iter() {
        let path_openlens_kubeconfigs = {
            let mut pathbuf = config_dir.clone();
            pathbuf.push(lens_compat_dir);
            pathbuf.push("kubeconfigs");

            pathbuf
        };

        match tokio::fs::read_dir(&path_openlens_kubeconfigs).await {
            Ok(mut openlens_kubeconfigs) => {
                while let Some(file) = openlens_kubeconfigs.next_entry().await.unwrap() {
                    let kubeconfig = Kubeconfig::read_from(file.path()).unwrap();
                    for context in &kubeconfig.contexts {
                        contexts.push(KubeContextSource {
                            provider: "file".to_owned(),
                            source: file.path().to_str().unwrap().to_owned(),
                            context: context.name.to_owned(),
                        });
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => {
                warn!(
                    "Failed to scan directory {:?} for kubeconfigs: {e}",
                    path_openlens_kubeconfigs
                )
            }
        }
    }

    Ok(contexts)
}

#[tauri::command]
#[tracing::instrument(skip_all, fields(request_id = tracing::field::Empty))]
pub async fn get_kubeconfig_yaml(context_source: KubeContextSource) -> Result<String, String> {
    crate::internal::tracing::set_span_request_id();

    if context_source.provider != "file" {
        return Err("Unsupport provider".to_owned());
    }

    let path = context_source.source;

    let file_contents = tokio::fs::read_to_string(path)
        .await
        .map_err(|e| e.to_string())?;

    Ok(file_contents)
}

#[tauri::command]
#[tracing::instrument(skip_all, fields(request_id = tracing::field::Empty))]
pub async fn write_kubeconfig_yaml(
    context_source: KubeContextSource,
    yaml: String,
) -> Result<(), String> {
    crate::internal::tracing::set_span_request_id();

    if context_source.provider != "file" {
        return Err("Unsupport provider".to_owned());
    }

    let path = context_source.source;

    tokio::fs::write(path, yaml)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}
