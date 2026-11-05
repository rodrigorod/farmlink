//! Parser for Farming Simulator 25 `modDesc.xml` descriptors.

use quick_xml::events::Event;
use quick_xml::reader::Reader;
use serde::{Deserialize, Serialize};

use crate::error::{ClientError, Result};

/// Extracted metadata from a Farming Simulator `modDesc.xml` file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModDesc {
    /// Friendly display title of the mod.
    pub title: String,

    /// Semantic or mod version (e.g., "1.2.0.0").
    pub version: String,

    /// Mod author or creator.
    pub author: String,

    /// FS description version (e.g., 90 for FS25).
    pub desc_version: Option<u32>,

    /// Path to mod icon file inside archive (e.g., "icon.dds" or "icon.png").
    pub icon_filename: Option<String>,

    /// Whether multiplayer is explicitly supported.
    pub multiplayer_supported: bool,
}

impl Default for ModDesc {
    fn default() -> Self {
        Self {
            title: String::new(),
            version: "1.0.0.0".to_string(),
            author: "Unknown".to_string(),
            desc_version: None,
            icon_filename: None,
            multiplayer_supported: true,
        }
    }
}

/// Parses a raw XML string into a structured [`ModDesc`].
///
/// Handles both simple text titles (`<title>My Mod</title>`)
/// and localized child elements (`<title><en>My Mod</en><fr>Mon Mod</fr></title>`).
pub fn parse_mod_desc(xml_content: &str) -> Result<ModDesc> {
    let mut reader = Reader::from_str(xml_content);
    reader.config_mut().trim_text(true);

    let mut mod_desc = ModDesc::default();
    let mut buf = Vec::new();

    // State tracking for nested tags
    let mut current_path: Vec<String> = Vec::new();

    // Map of localized titles: "en" -> "...", "fr" -> "...", "" -> direct text
    let mut localized_titles: Vec<(String, String)> = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_lowercase();

                if name == "moddesc" {
                    // Extract descVersion attribute if present
                    for attr in e.attributes().flatten() {
                        let key = String::from_utf8_lossy(attr.key.as_ref()).to_lowercase();
                        if key == "descversion" {
                            if let Ok(val_str) = String::from_utf8(attr.value.to_vec()) {
                                mod_desc.desc_version = val_str.trim().parse().ok();
                            }
                        }
                    }
                } else if name == "multiplayer" {
                    for attr in e.attributes().flatten() {
                        let key = String::from_utf8_lossy(attr.key.as_ref()).to_lowercase();
                        if key == "supported" {
                            let val_str = String::from_utf8_lossy(&attr.value).to_lowercase();
                            mod_desc.multiplayer_supported = val_str != "false";
                        }
                    }
                }

                current_path.push(name);
            }
            Ok(Event::End(_)) => {
                current_path.pop();
            }
            Ok(Event::Text(ref e)) => {
                let text = match e.unescape() {
                    Ok(cow) => cow.trim().to_string(),
                    Err(_) => String::from_utf8_lossy(e.as_ref()).trim().to_string(),
                };

                if !text.is_empty() {
                    let path_str = current_path.join("/");

                    if path_str == "moddesc/version" {
                        mod_desc.version = text;
                    } else if path_str == "moddesc/author" {
                        mod_desc.author = text;
                    } else if path_str == "moddesc/iconfilename" {
                        mod_desc.icon_filename = Some(text);
                    } else if path_str == "moddesc/title" {
                        // Direct title text
                        localized_titles.push((String::new(), text));
                    } else if current_path.len() == 3
                        && current_path[0] == "moddesc"
                        && current_path[1] == "title"
                    {
                        // Localized language code, e.g. "en", "fr", "de"
                        let lang = current_path[2].clone();
                        localized_titles.push((lang, text));
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(ClientError::Xml(e)),
            _ => {}
        }
        buf.clear();
    }

    // Select the best available title
    mod_desc.title = select_best_title(&localized_titles);

    Ok(mod_desc)
}

/// Chooses the most suitable title prioritizing English, French, German, or first non-empty.
fn select_best_title(titles: &[(String, String)]) -> String {
    if titles.is_empty() {
        return String::new();
    }

    // Check preferred languages
    for lang in &["en", "fr", "de", ""] {
        if let Some((_, text)) = titles.iter().find(|(l, _)| l.eq_ignore_ascii_case(lang)) {
            if !text.is_empty() {
                return text.clone();
            }
        }
    }

    // Fallback to first available non-empty
    titles.first().map(|(_, t)| t.clone()).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_standard_localized_mod_desc() {
        let xml = r#"<?xml version="1.0" encoding="utf-8" standalone="no" ?>
        <modDesc descVersion="90">
            <author>GIANTS Software</author>
            <version>1.2.0.0</version>
            <title>
                <en>John Deere 7R Series 2024</en>
                <fr>John Deere Série 7R 2024</fr>
                <de>John Deere 7R Serie 2024</de>
            </title>
            <iconFilename>icon_jd7r.dds</iconFilename>
            <multiplayer supported="true"/>
        </modDesc>"#;

        let res = parse_mod_desc(xml).expect("parse valid xml");
        assert_eq!(res.title, "John Deere 7R Series 2024");
        assert_eq!(res.version, "1.2.0.0");
        assert_eq!(res.author, "GIANTS Software");
        assert_eq!(res.desc_version, Some(90));
        assert_eq!(res.icon_filename, Some("icon_jd7r.dds".to_string()));
        assert!(res.multiplayer_supported);
    }

    #[test]
    fn parses_simple_flat_title() {
        let xml = r#"<modDesc>
            <author>Modder42</author>
            <version>2.0.1</version>
            <title>Courseplay NextGen</title>
        </modDesc>"#;

        let res = parse_mod_desc(xml).expect("parse valid flat xml");
        assert_eq!(res.title, "Courseplay NextGen");
        assert_eq!(res.version, "2.0.1");
        assert_eq!(res.author, "Modder42");
        assert_eq!(res.desc_version, None);
    }

    #[test]
    fn handles_empty_or_minimal_xml() {
        let xml = r#"<modDesc></modDesc>"#;
        let res = parse_mod_desc(xml).expect("parse empty modDesc");
        assert_eq!(res.version, "1.0.0.0");
        assert_eq!(res.author, "Unknown");
        assert_eq!(res.title, "");
    }
}
