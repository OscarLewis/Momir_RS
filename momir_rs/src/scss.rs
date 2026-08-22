use std::fs;

pub(crate) fn compile_scss() -> Result<(), Box<dyn std::error::Error>> {
    let css = grass::from_path("static/scss/style.scss", &grass::Options::default())?;

    fs::write("static/style.css", css)?;

    Ok(())
}

/*
fn watch_scss() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (tx, rx) = channel();

    let mut watcher = notify::recommended_watcher(tx)?;

    watcher.watch(Path::new("static/scss"), RecursiveMode::Recursive)?;

    for result in rx {
        match result {
            Ok(event) => {
                if event
                    .paths
                    .iter()
                    .any(|path| path.extension().is_some_and(|ext| ext == "scss"))
                {
                    tracing::debug!("SCSS changed");

                    if let Err(err) = compile_scss() {
                        tracing::error!("SCSS compilation failed: {err}");
                    }
                }
            }
            Err(err) => {
                tracing::error!("SCSS watcher error: {err}");
            }
        }
    }

    Ok(())
}
*/
