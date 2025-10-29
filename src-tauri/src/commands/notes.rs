use serde::{Deserialize, Serialize};
use std::fs;
use crate::config;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Note {
    #[serde(default)]
    pub game_id: Option<String>,
    #[serde(default)]
    pub game_name: Option<String>,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
}
// Legacy format support: older notes used `game_path`. We'll try to parse the
// current format first; if it fails, attempt to parse legacy entries and
// convert them so migration is seamless.
#[derive(Serialize, Deserialize, Debug, Clone)]
struct LegacyNote {
    #[serde(default)]
    pub game_path: Option<String>,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
}

fn load_all() -> Result<Vec<Note>, String> {
    let path = config::notes_path();
    if let Ok(s) = fs::read_to_string(&path) {
        // try current format
        if let Ok(v) = serde_json::from_str::<Vec<Note>>(&s) {
            return Ok(v);
        }
        // try legacy format and convert
        if let Ok(v2) = serde_json::from_str::<Vec<LegacyNote>>(&s) {
            let converted = v2
                .into_iter()
                .map(|ln| Note {
                    game_id: None,
                    game_name: None,
                    title: ln.title,
                    content: ln.content,
                    created_at: ln.created_at,
                    updated_at: ln.updated_at,
                })
                .collect();
            return Ok(converted);
        }
        Err(format!("parse notes: invalid format"))
    } else {
        Ok(Vec::new())
    }
}

fn save_all(notes: &Vec<Note>) -> Result<(), String> {
    let path = config::notes_path();
    if let Some(parent) = path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            return Err(format!("create notes dir: {}", e));
        }
    }
    let s = serde_json::to_string_pretty(notes).map_err(|e| e.to_string())?;
    let tmp = path.with_extension(format!("tmp.{}", std::process::id()));
    fs::write(&tmp, s).map_err(|e| e.to_string())?;
    fs::rename(&tmp, &path).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn list_notes() -> Result<Vec<Note>, String> {
    load_all()
}

#[tauri::command]
pub fn get_note(game_id: &str) -> Result<Option<Note>, String> {
    let notes = load_all()?;
    for n in notes.into_iter() {
        if let Some(gid) = &n.game_id {
            if gid == game_id {
                return Ok(Some(n));
            }
        }
    }
    Ok(None)
}

#[tauri::command]
pub fn save_note(note: Note) -> Result<Note, String> {
    let mut notes = load_all()?;
    let mut new_note = note.clone();

    // Ensure timestamps
    let now = chrono::Utc::now().to_rfc3339();
    if new_note.created_at.trim().is_empty() {
        new_note.created_at = now.clone();
    }
    new_note.updated_at = now.clone();

    // Upsert by game_id if present
    if let Some(gid) = &new_note.game_id {
        if !gid.trim().is_empty() {
            let mut found = false;
            for n in notes.iter_mut() {
                if let Some(existing_gid) = &n.game_id {
                    if existing_gid == gid {
                        n.title = new_note.title.clone();
                        n.content = new_note.content.clone();
                        n.game_name = new_note.game_name.clone();
                        n.updated_at = new_note.updated_at.clone();
                        new_note = n.clone();
                        found = true;
                        break;
                    }
                }
            }
            if !found {
                notes.push(new_note.clone());
            }
        } else {
            // empty game_id -> append as standalone note
            notes.push(new_note.clone());
        }
    } else {
        // no game_id -> append as standalone note
        notes.push(new_note.clone());
    }

    save_all(&notes)?;
    Ok(new_note)
}

#[tauri::command]
pub fn delete_note(game_id: &str) -> Result<(), String> {
    let mut notes = load_all()?;
    notes.retain(|n| match &n.game_id {
        Some(gid) => gid != game_id,
        None => true,
    });
    save_all(&notes)?;
    Ok(())
}

#[tauri::command]
pub fn delete_note_by_index(index: usize) -> Result<(), String> {
    let mut notes = load_all()?;
    if index >= notes.len() {
        return Err(format!("index out of range: {}", index));
    }
    notes.remove(index);
    save_all(&notes)?;
    Ok(())
}
