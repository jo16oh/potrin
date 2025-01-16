use eyre::eyre;
use std::any::type_name;
use tauri::async_runtime::RwLock;
use tauri::{Manager, Runtime, State};

pub async fn set_rw_state<R: Runtime, T>(manager: &impl Manager<R>, value: T) -> eyre::Result<()>
where
    T: 'static + Sync + Send,
{
    match manager.try_state::<RwLock<T>>() {
        Some(ref mut state) => {
            let mut state = state.write().await;
            *state = value;
        }
        None => {
            manager.manage(RwLock::new(value));
        }
    };
    Ok(())
}

pub fn get_rw_state<R: Runtime, T>(manager: &impl Manager<R>) -> eyre::Result<State<'_, RwLock<T>>>
where
    T: 'static + Sync + Send,
{
    manager
        .try_state::<RwLock<T>>()
        .ok_or_else(|| eyre!(format!("failed to get state {}", type_name::<RwLock<T>>())))
}

pub fn get_state<R: Runtime, T>(manager: &impl Manager<R>) -> eyre::Result<&T>
where
    T: 'static + Sync + Send,
{
    manager
        .try_state::<T>()
        .map(|r| r.inner())
        .ok_or_else(|| eyre!(format!("failed to get state {}", type_name::<T>())))
}

pub fn extract_text_from_doc(doc: &str) -> eyre::Result<String> {
    use serde::Deserialize;

    #[derive(Deserialize, Debug)]
    struct Document {
        #[serde(default)]
        content: Vec<Document>,
        #[serde(default)]
        text: Option<String>,
    }

    let document: Document = serde_json::from_str(doc)?;

    fn extract_text(document: &Document, result: &mut String) {
        if let Some(ref text) = document.text {
            result.push_str(text);
        }

        for item in &document.content {
            extract_text(item, result);
        }
    }

    let mut result = String::new();
    extract_text(&document, &mut result);

    Ok(result)
}

#[cfg(test)]
mod test {
    use super::extract_text_from_doc;

    #[test]
    fn test_extract_text_from_doc() {
        let doc = r#"
            {
                "type": "doc",
                "content": [
                  {
                    "type": "paragraph",
                    "content": [
                      {
                        "type": "text",
                        "text": "Example "
                      }
                    ]
                  },
                  {
                    "type": "paragraph"
                  },
                  {
                    "type": "paragraph",
                    "content": [
                      {
                        "type": "text",
                        "text": "Content"
                      },
                      {
                        "type": "other"
                      }
                    ]
                  }
                ]
            }
        "#;

        let result = extract_text_from_doc(doc).unwrap();
        assert_eq!(result, "Example Content".to_string());
    }
}
