use std::{io::ErrorKind, path::PathBuf};

use scan_dir::ScanDir;
use tauri::Manager as _;

/// A ScriptType determines which capabilties are available in the scripting engine
/// at runtime. This avoids unexpected side-effects of poorly-written scripts.
pub enum ScriptType {
    Menu,
}

pub struct ScriptsProvider {
    app: tauri::AppHandle,
}

impl ScriptsProvider {
    pub fn new(app: tauri::AppHandle) -> Self {
        ScriptsProvider { app }
    }

    fn list_packages_in(&self, dir: &PathBuf) -> Vec<PathBuf> {
        let mut packages: Vec<PathBuf> = Vec::new();

        let result = ScanDir::dirs().walk(dir, |iter| {
            let mut entrypoints = iter
                .map(|(entry, _)| entry.path())
                .collect::<Vec<PathBuf>>();

            packages.append(&mut entrypoints);
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

        packages
    }

    fn get_builtin_packages(&self) -> Result<Vec<PathBuf>, Error> {
        let builtins_base = self.app.path().resource_dir().map(|mut dir| {
            dir.push("scripts");
            dir
        })?;

        Ok(self.list_packages_in(&builtins_base))
    }

    fn get_extension_packages(&self) -> Result<Vec<PathBuf>, Error> {
        let extensions_base = self
            .app
            .path()
            .app_data_dir()
            .map(|mut dir| {
                dir.push("extensions");
                dir
            })
            .map_err(Error::TauriDir)?;

        Ok(self.list_packages_in(&extensions_base))
    }

    fn get_scripts_in_package(
        &self,
        mut package_path: PathBuf,
        script_type: &ScriptType,
    ) -> Vec<PathBuf> {
        let mut scripts: Vec<PathBuf> = Vec::new();

        match script_type {
            ScriptType::Menu => package_path.push("menus"),
        }

        let result = ScanDir::files().walk(package_path, |iter| {
            let mut entrypoints = iter
                .filter(|(_, name)| name.ends_with(".rhai"))
                .map(|(ref entry, _)| entry.path())
                .collect::<Vec<PathBuf>>();

            scripts.append(&mut entrypoints);
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

        scripts.sort();

        scripts
    }

    pub fn get_scripts_for_type(&self, script_type: ScriptType) -> Result<Vec<PathBuf>, Error> {
        let builtins = self.get_builtin_packages()?;
        let extensions = self.get_extension_packages()?;

        Ok(builtins
            .into_iter()
            .chain(extensions)
            .flat_map(|pkg| self.get_scripts_in_package(pkg, &script_type))
            .collect())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Error resolving directory: {0}")]
    TauriDir(#[from] tauri::Error),
}
