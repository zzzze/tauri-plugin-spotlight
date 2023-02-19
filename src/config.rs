use std::collections::HashMap;

#[derive(serde::Deserialize, Default, Debug, Clone, PartialEq)]
pub struct WindowConfig {
    pub label: String,
    pub shortcut: String,
}

#[derive(serde::Deserialize, Default, Debug, Clone, PartialEq)]
pub struct PluginConfig {
    pub windows: Option<Vec<WindowConfig>>,
    pub close_shortcut: Option<String>,
    pub hide_when_inactive: Option<bool>,
}

impl PluginConfig {
    pub fn merge(a: &Self, b: &Self) -> Self {
        let mut windows: Vec<WindowConfig> = vec![];
        if let Some(w) = a.windows.clone() {
            windows = w;
        } else if let Some(w) = b.windows.clone() {
            windows = w;
        }
        let mut dict: HashMap<String, String> = HashMap::default();
        for w in &windows {
            dict.insert(w.label.clone(), w.shortcut.clone());
        }
        if let Some(w) = b.windows.clone() {
            for config in w {
                if !dict.contains_key(&config.label) {
                    windows.push(WindowConfig { label: config.label, shortcut: config.shortcut });
                }
            }
        }
        Self {
            windows: {
                if windows.len() == 0 {
                    None
                } else {
                    Some(windows)
                }
            },
            close_shortcut: a.close_shortcut.clone().or(b.close_shortcut.clone()),
            hide_when_inactive: a.hide_when_inactive.clone().or(b.hide_when_inactive.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::WindowConfig;
    use super::PluginConfig;

    #[test]
    fn merge_and_override_default_value() {
        let a = PluginConfig::default();
        let b = PluginConfig {
            windows: Some(vec![
                WindowConfig {
                    label: String::from("main"),
                    shortcut: String::from("Ctrl+I"),
                },
            ]),
            close_shortcut: Some(String::from("Escape")),
            hide_when_inactive: Some(true),
        };
        let c = PluginConfig::merge(&a, &b);
        assert_eq!(c, b);
    }

    #[test]
    fn merge_windows() {
        let a = PluginConfig {
            windows: Some(vec![
                WindowConfig {
                    label: String::from("main"),
                    shortcut: String::from("Ctrl+I"),
                },
            ]),
            close_shortcut: None,
            hide_when_inactive: None,
        };
        let b = PluginConfig {
            windows: Some(vec![
                WindowConfig {
                    label: String::from("foo"),
                    shortcut: String::from("bar"),
                },
            ]),
            close_shortcut: None,
            hide_when_inactive: None,
        };
        let c = PluginConfig::merge(&a, &b);
        assert_eq!(c, PluginConfig {
            windows: Some(vec![
                WindowConfig {
                    label: String::from("main"),
                    shortcut: String::from("Ctrl+I"),
                },
                WindowConfig {
                    label: String::from("foo"),
                    shortcut: String::from("bar"),
                },
            ]),
            close_shortcut: None,
            hide_when_inactive: None,
        });
    }

    #[test]
    fn a_takes_precedence_over_b() {
        let a = PluginConfig {
            windows: None,
            close_shortcut: Some(String::from("Escape")),
            hide_when_inactive: Some(true),
        };
        let b = PluginConfig {
            windows: None,
            close_shortcut: Some(String::from("baz")),
            hide_when_inactive: Some(false),
        };
        let c = PluginConfig::merge(&a, &b);
        assert_eq!(c, a);
    }
}
