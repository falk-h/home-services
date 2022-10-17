use std::collections::HashMap;

#[cfg(debug_assertions)]
use std::sync::RwLock;

use axum::response::Html;
use rand::Rng;
use tera::{Context, Function, Tera, Value};

use crate::errors::Error;

#[derive(Debug)]
pub struct Renderer {
    #[cfg(debug_assertions)]
    tera: RwLock<Tera>,

    // No locking needed if we're not doing any hot reloading.
    #[cfg(not(debug_assertions))]
    tera: Tera,
}

impl Renderer {
    pub fn new(template_dir: &str) -> Result<Self, Error> {
        let mut tera = Tera::new(&format!("{template_dir}/**/*"))?;

        tracing::debug!(
            "Found templates in {template_dir}: {}",
            tera.get_template_names().fold(String::new(), |mut acc, s| {
                if !acc.is_empty() {
                    acc += ", ";
                }
                acc + s
            }),
        );

        tera.register_function("rand_f64", RandF64);

        #[cfg(debug_assertions)]
        let tera = RwLock::new(tera);

        Ok(Self { tera })
    }

    pub fn render(&self, template: &str, context: &Context) -> Result<String, Error> {
        let tera = {
            // Hot reload templates in debug builds.
            #[cfg(debug_assertions)]
            {
                tracing::trace!("Reloading Tera templates");
                self.tera.write().unwrap().full_reload()?;

                self.tera.read().unwrap()
            }

            #[cfg(not(debug_assertions))]
            &self.tera
        };

        Ok(tera.render(template, context)?)
    }

    pub fn render_html(&self, template: &str, context: &Context) -> Result<Html<Vec<u8>>, Error> {
        let s = self.render(template, context)?;
        let minified = minify_html::minify(s.as_bytes(), &Default::default());
        Ok(Html(minified))
    }

    pub fn render_error(&self, err: &Error) -> Result<Html<Vec<u8>>, String> {
        // TODO: Fancier errors with eyre.

        let mut context = Context::new();
        context.insert("error", &err.to_string());

        self.render_html("error.html", &context)
            .map_err(|tera_err| {
                format!(
                    "An error occured, but I failed to turn it into a nice HTML page: {tera_err}. \
                Here's the original error: {err}.",
                )
            })
    }
}

struct RandF64;

impl Function for RandF64 {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        let default_max = Value::from(1.0);
        let max = args.get("max").unwrap_or(&default_max);

        if let Value::Number(max) = max {
            if let Some(max) = max.as_f64() {
                let n = rand::thread_rng().gen_range::<f64, _>(0.0..max);
                return Ok(Value::from(n));
            }
        }

        Err(tera::Error::msg("`max` is not an f64"))
    }
}
