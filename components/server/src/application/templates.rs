use std::{path::PathBuf, sync::Arc};

use anyhow::Context;
use tera::Tera;

#[derive(Clone)]
pub struct Templates {
    inner: Arc<Tera>,
}

impl Templates {
    pub fn new(templates_folder: PathBuf) -> anyhow::Result<Self> {
        let _ = std::fs::read_dir(&templates_folder)
            .with_context(|| format!("invalid templates folder: {:?}", templates_folder))?;

        let templates = templates_folder.join("**").join("*.html");
        let tera = Tera::new(templates.to_str().context("invalid templates path")?)
            .context("Failed to load templates")?;
        Ok(Self {
            inner: Arc::new(tera),
        })
    }

    pub fn render(&self, template_name: &str, context: &tera::Context) -> anyhow::Result<String> {
        self.inner
            .render(template_name, context)
            .context("Failed to render template")
    }
}
