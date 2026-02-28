use std::{io::ErrorKind, path::PathBuf};

use scan_dir::ScanDir;
use tauri::Manager as _;

const SCRIPT_ENTRYPOINT_FILE: &str = "main.rhai";

pub struct ScriptsProvider {
    app: tauri::AppHandle,
}

impl ScriptsProvider {
    pub fn new(app: tauri::AppHandle) -> Self {
        ScriptsProvider { app }
    }

    /// Returns a list of paths to entrypoints of builtin (i.e. bundled with the application) scripts. Bundled scripts are not
    /// expected to be changed at runtime.
    ///
    /// Refer to [`tauri::path::PathResolver::resource_dir`] to understand how paths are resolved during development and at runtime.
    pub fn get_builtins_entrypoints(&self) -> Result<Vec<PathBuf>, Error> {
        let builtins_base = self
            .app
            .path()
            .resource_dir()
            .map(|mut dir| {
                dir.push("scripts");
                dir.push("builtin");
                dir
            })
            .map_err(Error::TauriDir)?;

        let mut entrypoint = builtins_base.clone();
        entrypoint.push(SCRIPT_ENTRYPOINT_FILE);

        Ok(Self::sanitize_nonexistent(vec![entrypoint]))
    }

    /// Returns a list of paths to entrypoints of user-provided scripts. These scripts might even be changed at runtime but there
    /// is neither a watching nor a reload mechanism yet.
    pub fn get_extensions_entrypoints(&self) -> Result<Vec<PathBuf>, Error> {
        let extensions_base = self
            .app
            .path()
            .app_data_dir()
            .map(|mut dir| {
                dir.push("extensions");
                dir
            })
            .map_err(Error::TauriDir)?;

        let mut extension_entrypoints: Vec<PathBuf> = Vec::new();

        let result = ScanDir::dirs().walk(&extensions_base, |iter| {
            let mut entrypoints = iter
                .map(|(entry, _)| entry.path())
                .map(|mut path| {
                    path.push(SCRIPT_ENTRYPOINT_FILE);
                    path
                })
                .collect::<Vec<PathBuf>>();

            extension_entrypoints.append(&mut entrypoints);
        });

        if let Err(errors) = result {
            let errors = errors.into_iter().filter(|e| match e {
                scan_dir::Error::Io(e, _) => e.kind() != ErrorKind::NotFound,
                scan_dir::Error::Decode(_) => true,
            });

            for e in errors {
                tracing::warn!("{e}");
            }
        }

        Ok(Self::sanitize_nonexistent(extension_entrypoints))
    }

    /// Removes files that do not exist.
    fn sanitize_nonexistent(mut files: Vec<PathBuf>) -> Vec<PathBuf> {
        files.retain(|file| {
            let exists = std::fs::exists(file).unwrap();

            if !exists {
                tracing::warn!(
                    "Entrypoint {} does not exist, ignoring.",
                    file.to_string_lossy()
                );
            }

            exists
        });

        files
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Error resolving directory: {0}")]
    TauriDir(tauri::Error),
}
