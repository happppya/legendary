//! Central registries (spec §3.2): canonical Epics/Disciplines taxonomy and
//! Landmark declarations with alias mapping, preventing path drift and
//! mass frontmatter churn on renames.

use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Canonical taxonomy: branch trees + legacy path aliases
/// (`.legend/taxonomy.yaml`).
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Taxonomy {
    #[serde(default)]
    pub disciplines: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub epics: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub aliases: HashMap<String, String>,
}

impl Taxonomy {
    /// Load `.legend/taxonomy.yaml`; a missing file yields an empty registry.
    pub fn load(legend_dir: &Path) -> io::Result<Self> {
        let path = legend_dir.join("taxonomy.yaml");
        match fs::read_to_string(&path) {
            Ok(text) => serde_yaml::from_str(&text).map_err(io::Error::other),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Taxonomy::default()),
            Err(e) => Err(e),
        }
    }

    /// Resolve a path through the alias map (aliases chain once).
    pub fn canonical_path(&self, path: &str) -> String {
        match self.aliases.get(path) {
            Some(canonical) => canonical.clone(),
            None => path.to_string(),
        }
    }

    /// Validate a hierarchical path (e.g. `Programming/Locomotion`) against a
    /// branch tree. A single segment is valid when it names a root branch
    /// (whole-domain assignment); a two-part path must resolve root → leaf.
    fn valid_in(tree: &BTreeMap<String, Vec<String>>, path: &str) -> bool {
        match path.split_once('/') {
            Some((root, leaf)) => tree
                .get(root)
                .is_some_and(|leaves| leaves.iter().any(|l| l == leaf)),
            None => tree.contains_key(path),
        }
    }

    pub fn is_valid_discipline(&self, path: &str) -> bool {
        Self::valid_in(&self.disciplines, &self.canonical_path(path))
    }

    pub fn is_valid_epic(&self, path: &str) -> bool {
        Self::valid_in(&self.epics, &self.canonical_path(path))
    }

    /// All valid paths in a branch tree as `root` and `root/leaf` strings
    /// (used for human-readable validation suggestions).
    fn flat_paths(tree: &BTreeMap<String, Vec<String>>) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for (root, leaves) in tree {
            out.push(root.clone());
            for leaf in leaves {
                out.push(format!("{root}/{leaf}"));
            }
        }
        out.sort();
        out
    }

    /// Valid discipline paths (`Programming`, `Programming/Locomotion`, ...).
    pub fn discipline_paths(&self) -> Vec<String> {
        Self::flat_paths(&self.disciplines)
    }

    /// Valid epic paths (`Combat_Engine`, `Combat_Engine/Locomotion`, ...).
    pub fn epic_paths(&self) -> Vec<String> {
        Self::flat_paths(&self.epics)
    }
}

/// One declared milestone / build checkpoint.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct LandmarkEntry {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub target_date: Option<String>,
}

/// Landmark registry: explicit declarations plus files auto-extracted from a
/// realm's `Landmarks/` folder, plus alias mapping.
#[derive(Debug, Clone, Default)]
pub struct LandmarkRegistry {
    pub landmarks: BTreeMap<String, LandmarkEntry>,
    pub aliases: HashMap<String, String>,
}

impl LandmarkRegistry {
    /// Load `.legend/landmarks.yaml` and merge landmarks auto-extracted from
    /// every `*.md` file in `landmarks_dir` (spec §3.2 B). A missing registry
    /// file yields an empty base set.
    pub fn load(legend_dir: &Path, landmarks_dir: &Path) -> io::Result<Self> {
        let mut reg = LandmarkRegistry::default();
        let file = legend_dir.join("landmarks.yaml");
        match fs::read_to_string(&file) {
            Ok(text) => {
                let doc: LandmarksDoc = serde_yaml::from_str(&text).map_err(io::Error::other)?;
                reg.landmarks = doc.landmarks;
                reg.aliases = doc.aliases;
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => return Err(e),
        }
        // Auto-extraction: any Markdown file inside `Landmarks/` declares a
        // landmark named after its file stem.
        if let Ok(entries) = fs::read_dir(landmarks_dir) {
            let mut stems: Vec<PathBuf> = entries
                .filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| p.extension().is_some_and(|x| x == "md"))
                .collect();
            stems.sort();
            for p in stems {
                if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
                    reg.landmarks
                        .entry(stem.to_string())
                        .or_insert_with(LandmarkEntry::default);
                }
            }
        }
        Ok(reg)
    }

    pub fn canonical(&self, id: &str) -> String {
        match self.aliases.get(id) {
            Some(canonical) => canonical.clone(),
            None => id.to_string(),
        }
    }

    pub fn is_valid(&self, id: &str) -> bool {
        self.landmarks.contains_key(&self.canonical(id))
    }

    /// Declared landmark names, sorted (explicit registry entries plus
    /// `Landmarks/` auto-extraction).
    pub fn landmark_names(&self) -> Vec<String> {
        self.landmarks.keys().cloned().collect()
    }
}

#[derive(Debug, Deserialize, Default)]
struct LandmarksDoc {
    #[serde(default)]
    landmarks: BTreeMap<String, LandmarkEntry>,
    #[serde(default)]
    aliases: HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_taxonomy_yaml() -> &'static str {
        "disciplines:\n  Programming:\n    - Locomotion\n    - Input\n  Art:\n    - UI\nepics:\n  Combat_Engine:\n    - Locomotion\naliases:\n  \"Programming/Movement\": \"Programming/Locomotion\"\n"
    }

    #[test]
    fn taxonomy_validation_and_aliases() {
        let tax: Taxonomy = serde_yaml::from_str(sample_taxonomy_yaml()).unwrap();
        assert!(tax.is_valid_discipline("Programming/Locomotion"));
        assert!(
            tax.is_valid_discipline("Programming/Movement"),
            "alias resolves"
        );
        assert!(!tax.is_valid_discipline("Programming/Netcode"));
        assert!(tax.is_valid_discipline("Art/UI"));
        assert!(tax.is_valid_epic("Combat_Engine/Locomotion"));
        assert!(!tax.is_valid_epic("Combat_Engine/Netcode"));
        assert_eq!(
            tax.canonical_path("Programming/Movement"),
            "Programming/Locomotion"
        );
    }

    #[test]
    fn flat_path_listing_sorted() {
        let tax: Taxonomy = serde_yaml::from_str(sample_taxonomy_yaml()).unwrap();
        let expected = vec![
            "Art".to_string(),
            "Art/UI".to_string(),
            "Programming".to_string(),
            "Programming/Input".to_string(),
            "Programming/Locomotion".to_string(),
        ];
        assert_eq!(tax.discipline_paths(), expected);
        assert_eq!(
            tax.epic_paths(),
            vec![
                "Combat_Engine".to_string(),
                "Combat_Engine/Locomotion".to_string()
            ]
        );
    }

    #[test]
    fn landmark_aliases_and_auto_extraction() {
        let dir = std::env::temp_dir().join("legend_test_landmarks");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join(".legend")).unwrap();
        fs::create_dir_all(dir.join("Landmarks")).unwrap();
        fs::write(
            dir.join(".legend").join("landmarks.yaml"),
            "landmarks:\n  Landmark_01_Demo:\n    title: Steam Playtest Demo\n    target_date: \"2026-11-15\"\naliases:\n  Demo_v1: Landmark_01_Demo\n",
        )
        .unwrap();
        fs::write(dir.join("Landmarks").join("Landmark_02_Slice.md"), "# x").unwrap();

        let reg = LandmarkRegistry::load(&dir.join(".legend"), &dir.join("Landmarks")).unwrap();
        assert!(reg.is_valid("Landmark_01_Demo"));
        assert!(reg.is_valid("Demo_v1"), "alias resolves");
        assert!(
            reg.is_valid("Landmark_02_Slice"),
            "auto-extracted from Landmarks/"
        );
        assert_eq!(reg.canonical("Demo_v1"), "Landmark_01_Demo");
        assert!(!reg.is_valid("Landmark_99"));
        let _ = fs::remove_dir_all(&dir);
    }
}
