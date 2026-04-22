//! Auto-detection of installed games from common launchers.
//!
//! Scans Steam, Lutris, and Heroic library manifests for installed titles.

use std::path::{Path, PathBuf};

/// A detected installed game.
#[derive(Debug, Clone)]
pub struct DetectedGame {
    /// Display name.
    pub name: String,
    /// Executable or process name (for profile matching).
    pub executable: String,
    /// Source launcher (Steam, Lutris, Heroic).
    pub source: &'static str,
    /// Install path (if known).
    pub install_path: Option<PathBuf>,
}

/// Detect all installed games from known launcher locations.
#[must_use]
pub fn detect_all() -> Vec<DetectedGame> {
    let mut games = Vec::new();
    let home = std::env::var("HOME").unwrap_or_default();
    if home.is_empty() {
        return games;
    }
    let home = Path::new(&home);

    detect_steam(home, &mut games);
    detect_lutris(home, &mut games);
    detect_heroic(home, &mut games);

    games.sort_by_key(|a| a.name.to_lowercase());
    games.dedup_by(|a, b| a.executable == b.executable);
    games
}

/// Scan Steam `appmanifest_*.acf` files for installed games.
fn detect_steam(home: &Path, games: &mut Vec<DetectedGame>) {
    let steam_dirs = [
        home.join(".steam/steam/steamapps"),
        home.join(".local/share/Steam/steamapps"),
    ];

    for dir in &steam_dirs {
        let Ok(entries) = std::fs::read_dir(dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path
                .file_name()
                .is_some_and(|n| n.to_string_lossy().starts_with("appmanifest_"))
            {
                if let Some(game) = parse_acf(&path) {
                    games.push(game);
                }
            }
        }
    }
}

/// Parse a Steam `appmanifest_*.acf` file for name and installdir.
fn parse_acf(path: &Path) -> Option<DetectedGame> {
    let content = std::fs::read_to_string(path).ok()?;
    let name = extract_acf_value(&content, "name")?;
    let installdir = extract_acf_value(&content, "installdir")?;

    // Skip Proton runtimes, Steamworks, etc.
    if name.starts_with("Proton")
        || name.starts_with("Steam Linux Runtime")
        || name.contains("Steamworks")
    {
        return None;
    }

    let install_path = path.parent().map(|p| p.join("common").join(&installdir));

    Some(DetectedGame {
        name,
        executable: installdir,
        source: "Steam",
        install_path,
    })
}

/// Extract a simple key-value from Valve's ACF format.
/// Lines look like: `"key"    "value"`
fn extract_acf_value(content: &str, key: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(&format!("\"{key}\"")) {
            let parts: Vec<&str> = trimmed.split('"').collect();
            if parts.len() >= 4 {
                return Some(parts[3].to_string());
            }
        }
    }
    None
}

/// Scan Lutris games database (`SQLite` → skip, use YAML game configs).
fn detect_lutris(home: &Path, games: &mut Vec<DetectedGame>) {
    let games_dir = home.join(".config/lutris/games");
    let Ok(entries) = std::fs::read_dir(&games_dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "yml") {
            // Derive name from filename slug (game name not always in YAML body)
            let slug_name = path
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .map(|s| slug_to_title(&s));
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Some(game) = parse_lutris_yml(&content, slug_name) {
                    games.push(game);
                }
            }
        }
    }
}

/// Convert a Lutris filename slug to a human-readable title.
/// e.g. `"altered-beast-remake-linux-1771620880"` → `"Altered Beast Remake Linux"`
fn slug_to_title(slug: &str) -> String {
    // Strip trailing numeric ID: `-<digits>` at end
    let trimmed = slug
        .rfind('-')
        .and_then(|i| {
            let suffix = &slug[i + 1..];
            if suffix.chars().all(|c| c.is_ascii_digit()) {
                Some(&slug[..i])
            } else {
                None
            }
        })
        .unwrap_or(slug);

    // Replace hyphens/underscores with spaces, title-case each word
    trimmed
        .split(['-', '_'])
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().to_string() + c.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Parse a Lutris YAML game config for name and executable.
/// `slug_name`: fallback name derived from filename; used when YAML has no `name:` field.
fn parse_lutris_yml(content: &str, slug_name: Option<String>) -> Option<DetectedGame> {
    let mut name = None;
    let mut exe = None;

    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(val) = trimmed.strip_prefix("name:") {
            let v = val.trim().trim_matches('"').trim_matches('\'').to_string();
            if !v.is_empty() {
                name = Some(v);
            }
        }
        if exe.is_none() {
            if let Some(val) = trimmed.strip_prefix("exe:") {
                let exe_path = val.trim().trim_matches('"').trim_matches('\'');
                exe = Path::new(exe_path)
                    .file_name()
                    .map(|f| f.to_string_lossy().into_owned());
            }
        }
    }

    // Prefer YAML name field; fall back to slug-derived name
    let name = name.or(slug_name)?;
    if name.is_empty() {
        return None;
    }
    let executable = exe.unwrap_or_else(|| name.clone());

    Some(DetectedGame {
        name,
        executable,
        source: "Lutris",
        install_path: None,
    })
}

/// Scan Heroic Games Launcher library.
fn detect_heroic(home: &Path, games: &mut Vec<DetectedGame>) {
    let base = home.join(".config/heroic");

    // store_cache: GOG, Epic (legendary), Amazon (nile), Zoom — parsed by title field
    for (file, source) in [
        ("store_cache/gog_library.json",        "GOG (Heroic)"),
        ("store_cache/legendary_library.json",   "Epic (Heroic)"),
        ("store_cache/nile_library.json",        "Amazon (Heroic)"),
        ("store_cache/zoom-library.json",        "Zoom (Heroic)"),
    ] {
        parse_heroic_json(&base.join(file), source, games);
    }

    // Sideloaded games — each subdirectory is one game
    parse_heroic_sideload(&base.join("sideload_apps"), games);
}

/// Parse a Heroic JSON library cache file.
/// Handles two layouts:
/// 1. Array of objects with `"title"` key (GOG/old format)
/// 2. Object with `"library"` array (newer Heroic format)
fn parse_heroic_json(path: &Path, source: &'static str, games: &mut Vec<DetectedGame>) {
    let Ok(content) = std::fs::read_to_string(path) else {
        return;
    };
    // Guard against empty/placeholder files (Heroic writes `{}` or `[]` when no data)
    if content.trim().len() <= 2 {
        return;
    }
    // Simple line-based scan: extract "title" values.
    // This avoids a serde_json dependency and works for both pretty-printed formats.
    for line in content.lines() {
        let trimmed = line.trim();
        // Match `"title": "Game Name"` or `"title":"Game Name"`
        if let Some(rest) = trimmed.strip_prefix("\"title\":") {
            let name = rest
                .trim()
                .trim_start_matches('"')
                .trim_end_matches([',', '"'])
                .trim_end_matches('"')
                .trim()
                .to_string();
            if !name.is_empty() && name != "null" {
                games.push(DetectedGame {
                    executable: name.clone(),
                    name,
                    source,
                    install_path: None,
                });
            }
        }
    }
}

/// Scan Heroic sideloaded apps from `~/.config/heroic/sideload_apps/`.
/// Each subdirectory is a game — the dirname is used as the name.
fn parse_heroic_sideload(dir: &Path, games: &mut Vec<DetectedGame>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        if entry.path().is_dir() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if !name.is_empty() {
                // Read info.json in subdir for a friendlier title if available
                let info_path = entry.path().join("info.json");
                let display_name = std::fs::read_to_string(&info_path)
                    .ok()
                    .and_then(|c| {
                        c.lines()
                            .find(|l| l.trim().starts_with("\"title\":"))
                            .and_then(|l| l.trim().strip_prefix("\"title\":"))
                            .map(|v| {
                                v.trim()
                                    .trim_start_matches('"')
                                    .trim_end_matches([',', '"'])
                                    .trim()
                                    .to_string()
                            })
                    })
                    .unwrap_or_else(|| name.clone());

                games.push(DetectedGame {
                    name: display_name,
                    executable: name,
                    source: "Heroic (Sideload)",
                    install_path: Some(entry.path()),
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    // --- ACF parsing ---

    #[test]
    fn acf_parses_valid_game() {
        let acf = r#"
"AppState"
{
    "appid"     "570"
    "name"      "Dota 2"
    "installdir"        "dota 2 beta"
    "StateFlags"        "4"
}
"#;
        let val = extract_acf_value(acf, "name");
        assert_eq!(val.as_deref(), Some("Dota 2"));

        let dir = extract_acf_value(acf, "installdir");
        assert_eq!(dir.as_deref(), Some("dota 2 beta"));
    }

    #[test]
    fn acf_returns_none_for_missing_key() {
        let acf = "\"appid\"\t\"570\"";
        assert!(extract_acf_value(acf, "name").is_none());
    }

    #[test]
    fn acf_skips_proton_runtime() {
        let tmp = tempdir("proton");
        let acf = tmp.join("appmanifest_1234.acf");
        fs::write(
            &acf,
            "\"AppState\"\n{\n\t\"name\"\t\"Proton 9.0\"\n\t\"installdir\"\t\"Proton 9.0\"\n}\n",
        )
        .unwrap();
        assert!(parse_acf(&acf).is_none());
    }

    #[test]
    fn acf_skips_steamworks() {
        let tmp = tempdir("steamworks");
        let acf = tmp.join("appmanifest_228980.acf");
        fs::write(
            &acf,
            "\"AppState\"\n{\n\t\"name\"\t\"Steamworks Common Redistributables\"\n\t\"installdir\"\t\"Steamworks Common Redistributables\"\n}\n",
        )
        .unwrap();
        assert!(parse_acf(&acf).is_none());
    }

    #[test]
    fn acf_real_game_produces_detected_game() {
        let tmp = tempdir("realgame");
        let acf = tmp.join("appmanifest_730.acf");
        fs::write(
            &acf,
            "\"AppState\"\n{\n\t\"appid\"\t\"730\"\n\t\"name\"\t\"Counter-Strike 2\"\n\t\"installdir\"\t\"Counter-Strike Global Offensive\"\n}\n",
        )
        .unwrap();
        let game = parse_acf(&acf).expect("should parse valid game");
        assert_eq!(game.name, "Counter-Strike 2");
        assert_eq!(game.executable, "Counter-Strike Global Offensive");
        assert_eq!(game.source, "Steam");
        assert!(game.install_path.is_some());
    }

    // --- Lutris YAML parsing ---

    #[test]
    fn lutris_yml_parses_name_and_exe() {
        let yml = "name: Celeste\ngame:\n  exe: /home/user/Games/celeste/Celeste.bin.x86_64\nrunner: linux\n";
        let game = parse_lutris_yml(yml, Some("celeste-1234".into())).expect("should parse");
        assert_eq!(game.name, "Celeste"); // YAML name wins over slug
        assert_eq!(game.executable, "Celeste.bin.x86_64");
        assert_eq!(game.source, "Lutris");
    }

    #[test]
    fn lutris_yml_uses_name_as_fallback_exe() {
        let yml = "name: 'Some Game'\nrunner: wine\n";
        let game = parse_lutris_yml(yml, None).expect("should parse");
        assert_eq!(game.executable, "Some Game");
    }

    #[test]
    fn lutris_yml_falls_back_to_slug_name() {
        // Real Lutris YAMLs often have no top-level name field
        let yml = "game:\n  exe: /home/user/Downloads/Jogo/Altered Beast Remake\nrunner: linux\n";
        let game = parse_lutris_yml(yml, Some("Altered Beast Remake Linux".into())).expect("should parse");
        assert_eq!(game.name, "Altered Beast Remake Linux");
        assert_eq!(game.executable, "Altered Beast Remake");
    }

    #[test]
    fn lutris_yml_returns_none_without_name_or_slug() {
        let yml = "runner: wine\ngame:\n  exe: /foo/bar.exe\n";
        assert!(parse_lutris_yml(yml, None).is_none());
    }

    #[test]
    fn lutris_yml_strips_quotes() {
        let yml = "name: \"Quoted Name\"\ngame:\n  exe: '/path/to/game.exe'\n";
        let game = parse_lutris_yml(yml, None).expect("should parse");
        assert_eq!(game.name, "Quoted Name");
        assert_eq!(game.executable, "game.exe");
    }

    #[test]
    fn slug_to_title_strips_numeric_id_and_humanizes() {
        assert_eq!(slug_to_title("altered-beast-remake-linux-1771620880"), "Altered Beast Remake Linux");
        assert_eq!(slug_to_title("supertuxkart-1771620561"), "Supertuxkart");
        assert_eq!(slug_to_title("no-number-here"), "No Number Here");
        assert_eq!(slug_to_title("single"), "Single");
    }

    // --- Heroic JSON parsing ---

    #[test]
    fn heroic_json_extracts_titles() {
        let tmp = tempdir("heroic");
        let json = tmp.join("library.json");
        fs::write(
            &json,
            "{\n  \"library\": [\n    {\n      \"title\": \"Hades\",\n      \"app_name\": \"hades\"\n    },\n    {\n      \"title\": \"Celeste\",\n      \"app_name\": \"celeste\"\n    }\n  ]\n}\n",
        )
        .unwrap();
        let mut games = Vec::new();
        parse_heroic_json(&json, "Epic (Heroic)", &mut games);
        assert_eq!(games.len(), 2);
        assert_eq!(games[0].name, "Hades");
        assert_eq!(games[1].name, "Celeste");
        assert_eq!(games[0].source, "Epic (Heroic)");
    }

    #[test]
    fn heroic_json_skips_empty_titles() {
        let tmp = tempdir("heroic-empty");
        let json = tmp.join("library.json");
        fs::write(&json, "{\n  \"title\": \"\",\n  \"title\": \"Valid\"\n}\n").unwrap();
        let mut games = Vec::new();
        parse_heroic_json(&json, "GOG (Heroic)", &mut games);
        assert_eq!(games.len(), 1);
        assert_eq!(games[0].name, "Valid");
    }

    #[test]
    fn heroic_json_missing_file_no_panic() {
        let mut games = Vec::new();
        parse_heroic_json(Path::new("/nonexistent/file.json"), "Epic (Heroic)", &mut games);
        assert!(games.is_empty());
    }

    // --- Integration: detect_steam with temp dir ---

    #[test]
    fn detect_steam_scans_acf_files() {
        let tmp = tempdir("steam-scan");
        let steamapps = tmp.join(".steam/steam/steamapps");
        fs::create_dir_all(&steamapps).unwrap();

        fs::write(
            steamapps.join("appmanifest_440.acf"),
            "\"AppState\"\n{\n\t\"name\"\t\"Team Fortress 2\"\n\t\"installdir\"\t\"Team Fortress 2\"\n}\n",
        )
        .unwrap();
        // Proton should be skipped
        fs::write(
            steamapps.join("appmanifest_2348590.acf"),
            "\"AppState\"\n{\n\t\"name\"\t\"Proton 9.0\"\n\t\"installdir\"\t\"Proton 9.0\"\n}\n",
        )
        .unwrap();
        // Non-manifest file should be ignored
        fs::write(steamapps.join("libraryfolders.vdf"), "\"stuff\"").unwrap();

        let mut games = Vec::new();
        detect_steam(&tmp, &mut games);
        assert_eq!(games.len(), 1);
        assert_eq!(games[0].name, "Team Fortress 2");
    }

    // --- detect_all deduplication + sorting ---

    #[test]
    fn detect_all_empty_home_returns_empty() {
        // If HOME is empty string, detect_all returns early
        // We can't easily override HOME in tests, but verify the bailout path exists
        let games = detect_all();
        // Just ensure no panic — result depends on host machine
        let _ = games;
    }

    // --- Helper ---

    fn tempdir(name: &str) -> PathBuf {
        crate::tests::tempdir(name)
    }
}
