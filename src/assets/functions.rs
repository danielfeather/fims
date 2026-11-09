use std::fmt::Write;

use minijinja::{State, Value};
use tracing::debug;

pub fn asset(entry_points: String, state: &State) -> String {
    debug!("Asssets ");

    let entry_points: Vec<&str> = entry_points.split(",").collect();

    let mut output = String::new();

    let Some(manifest) = state.lookup("manifest") else {
        let _ = write!(
            &mut output,
            r#"<script type="module" async src="http://localhost:5173/@vite/client"></script>
            "#
        );

        for entry_point in entry_points {
            let _ = write!(
                &mut output,
                r#"<script type="module" async src="http://localhost:5173/{entry_point}"></script>
                "#
            );
        }
        return output;
    };

    for entry_point in entry_points {
        let Ok(entry) = manifest.get_item(&Value::from(entry_point)) else {
            return output;
        };

        let Ok(file) = entry.get_item(&Value::from("file")) else {
            return output;
        };

        let Some(file) = file.to_str() else {
            return output;
        };

        if let Ok(stylesheets) = entry.get_item(&Value::from("css")) {
            let Ok(stylesheets) = stylesheets.try_iter() else {
                return output;
            };

            for stylesheet in stylesheets {
                let Some(stylesheet) = stylesheet.to_str() else {
                    continue;
                };

                let _ = write!(
                    &mut output,
                    r#"<link href="/{stylesheet}" rel="stylesheet">
                "#,
                );
            }
        }

        let _ = write!(
            &mut output,
            r#"<script type="module" async src="/{file}"></script>
            "#,
        );
    }

    output
}
