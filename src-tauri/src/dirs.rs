use std::path::PathBuf;

fn get_config_dir() -> Option<PathBuf> {
    let config_dir = dirs::config_dir();

    if config_dir.is_none() {
        eprintln!("Cannot scan config dir for custom views because its location is not known. This might be an unsupported platform.");
        return None;
    }

    let mut config_dir = config_dir.unwrap();
    config_dir.push("hyprkube");

    Some(config_dir)
}

pub fn get_views_dir() -> Option<PathBuf> {
    let mut views_dir = get_config_dir()?;
    views_dir.push("views");

    if !views_dir.exists() {
        match std::fs::create_dir_all(&views_dir) {
            Ok(()) => (),
            Err(error) => {
                eprintln!(
                    "Failed to create directory {:?} for view scripts: {:?}",
                    views_dir, error
                );
                return None;
            }
        }
    }

    Some(views_dir)
}
