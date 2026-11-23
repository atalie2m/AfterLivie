use anyhow::Context;
use chrono::Utc;
use replay_core::{
    global_comment_key, CommentEvent, Diagnostic, RenderHistoryEntry, ReplayProject,
    ReproducibilityManifest,
};
use rusqlite::{params, Connection};
use std::fs;
use std::path::{Path, PathBuf};

pub const PROJECT_EXTENSION: &str = "replayproj";

#[derive(Debug, Clone)]
pub struct ProjectPackage {
    pub root: PathBuf,
}

impl ProjectPackage {
    pub fn create(root: impl AsRef<Path>, project: &ReplayProject) -> anyhow::Result<Self> {
        let root = root.as_ref();
        create_project_dirs(root)?;
        let package = Self {
            root: root.to_path_buf(),
        };
        package.save_project(project)?;
        Ok(package)
    }

    pub fn open(root: impl AsRef<Path>) -> anyhow::Result<Self> {
        let root = root.as_ref();
        if !root.is_dir() {
            anyhow::bail!("project package does not exist: {}", root.display());
        }
        Ok(Self {
            root: root.to_path_buf(),
        })
    }

    pub fn save_project(&self, project: &ReplayProject) -> anyhow::Result<()> {
        let mut project = project.clone();
        project.updated_at = Utc::now();
        let path = self.root.join("project.json");
        atomic_write_json(&path, &project)
    }

    pub fn load_project(&self) -> anyhow::Result<ReplayProject> {
        let path = self.root.join("project.json");
        let bytes = fs::read(&path).with_context(|| format!("read {}", path.display()))?;
        serde_json::from_slice(&bytes).with_context(|| format!("parse {}", path.display()))
    }

    pub fn save_comments(&self, comments: &[CommentEvent]) -> anyhow::Result<()> {
        let db_path = self.root.join("comments.sqlite");
        let mut connection = Connection::open(db_path)?;
        create_comments_schema(&connection)?;
        let tx = connection.transaction()?;
        for comment in comments {
            tx.execute(
                "INSERT OR ABORT INTO comments (
                  global_key, source_id, comment_id, timestamp_ms, kind, author_json, body_json,
                  style_json, badges_json, emotes_json, raw_ref_json, import_order
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                params![
                    global_comment_key(&comment.source_id, &comment.id),
                    comment.source_id,
                    comment.id,
                    comment.timestamp_ms,
                    serde_json::to_string(&comment.kind)?,
                    serde_json::to_string(&comment.author)?,
                    serde_json::to_string(&comment.body)?,
                    serde_json::to_string(&comment.style)?,
                    serde_json::to_string(&comment.badges)?,
                    serde_json::to_string(&comment.emotes)?,
                    serde_json::to_string(&comment.raw_ref)?,
                    comment.import_order as i64,
                ],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn load_comments(&self) -> anyhow::Result<Vec<CommentEvent>> {
        let db_path = self.root.join("comments.sqlite");
        let connection = Connection::open(db_path)?;
        create_comments_schema(&connection)?;
        let mut statement = connection.prepare(
            "SELECT source_id, comment_id, timestamp_ms, kind, author_json, body_json,
                    style_json, badges_json, emotes_json, raw_ref_json, import_order
             FROM comments
             ORDER BY timestamp_ms, import_order, comment_id",
        )?;
        let rows = statement.query_map([], |row| {
            let source_id: String = row.get(0)?;
            let id: String = row.get(1)?;
            let kind_json: String = row.get(3)?;
            let author_json: String = row.get(4)?;
            let body_json: String = row.get(5)?;
            let style_json: String = row.get(6)?;
            let badges_json: String = row.get(7)?;
            let emotes_json: String = row.get(8)?;
            let raw_ref_json: String = row.get(9)?;
            let import_order: i64 = row.get(10)?;
            Ok(CommentEvent {
                id,
                source_id,
                platform: None,
                timestamp_ms: row.get(2)?,
                original_timestamp: None,
                kind: serde_json::from_str(&kind_json).map_err(json_to_sql_error)?,
                author: serde_json::from_str(&author_json).map_err(json_to_sql_error)?,
                body: serde_json::from_str(&body_json).map_err(json_to_sql_error)?,
                style: serde_json::from_str(&style_json).map_err(json_to_sql_error)?,
                badges: serde_json::from_str(&badges_json).map_err(json_to_sql_error)?,
                emotes: serde_json::from_str(&emotes_json).map_err(json_to_sql_error)?,
                source: None,
                raw_ref: serde_json::from_str(&raw_ref_json).map_err(json_to_sql_error)?,
                import_order: import_order.max(0) as u64,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn save_diagnostics(&self, diagnostics: &[Diagnostic]) -> anyhow::Result<()> {
        atomic_write_json(&self.root.join("diagnostics.json"), diagnostics)
    }

    pub fn save_render_history(&self, history: &[RenderHistoryEntry]) -> anyhow::Result<()> {
        atomic_write_json(&self.root.join("render-history.json"), history)
    }

    pub fn save_reproducibility(&self, manifest: &ReproducibilityManifest) -> anyhow::Result<()> {
        atomic_write_json(&self.root.join("reproducibility.json"), manifest)
    }
}

pub fn create_project_dirs(root: &Path) -> anyhow::Result<()> {
    let dirs = [
        "",
        "sources",
        "assets",
        "assets/avatars",
        "assets/badges",
        "assets/emotes",
        "assets/backgrounds",
        "assets/fonts",
        "templates",
        "templates/custom-layouts",
        "cache",
        "cache/thumbnails",
        "cache/waveform",
        "cache/preview-renders",
        "cache/overlay-cache",
        "cache/media-proxies",
        "cache/intermediates",
    ];
    for dir in dirs {
        fs::create_dir_all(root.join(dir))?;
    }
    Ok(())
}

fn create_comments_schema(connection: &Connection) -> anyhow::Result<()> {
    connection.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS schema_version (
          component TEXT PRIMARY KEY,
          version INTEGER NOT NULL
        );
        INSERT OR IGNORE INTO schema_version (component, version) VALUES ('comments', 1);

        CREATE TABLE IF NOT EXISTS comments (
          global_key TEXT PRIMARY KEY,
          source_id TEXT NOT NULL,
          comment_id TEXT NOT NULL,
          timestamp_ms INTEGER NOT NULL,
          kind TEXT NOT NULL,
          author_json TEXT,
          body_json TEXT NOT NULL,
          style_json TEXT,
          badges_json TEXT,
          emotes_json TEXT,
          raw_ref_json TEXT,
          import_order INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_comments_time ON comments(timestamp_ms);
        CREATE INDEX IF NOT EXISTS idx_comments_source_time ON comments(source_id, timestamp_ms);
        ",
    )?;
    Ok(())
}

fn atomic_write_json<T: serde::Serialize + ?Sized>(path: &Path, value: &T) -> anyhow::Result<()> {
    let tmp_path = path.with_extension("tmp");
    let bytes = serde_json::to_vec_pretty(value)?;
    fs::write(&tmp_path, bytes)?;
    fs::rename(&tmp_path, path)?;
    Ok(())
}

fn json_to_sql_error(error: serde_json::Error) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(error))
}

#[cfg(test)]
mod tests {
    use super::*;
    use replay_core::{CommentBody, CommentEvent, CommentKind, ReplayProject};

    #[test]
    fn project_package_roundtrips_project_and_comments() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("Sample.replayproj");
        let project = ReplayProject::new();
        let package = ProjectPackage::create(&root, &project).unwrap();
        let loaded = package.load_project().unwrap();
        assert_eq!(loaded.project_id, project.project_id);

        let comment = CommentEvent {
            id: "c1".into(),
            source_id: "main".into(),
            platform: None,
            timestamp_ms: 1000,
            original_timestamp: None,
            kind: CommentKind::Message,
            author: None,
            body: CommentBody {
                text: "hello".into(),
            },
            style: None,
            badges: vec![],
            emotes: vec![],
            source: None,
            raw_ref: None,
            import_order: 0,
        };
        package.save_comments(&[comment]).unwrap();
        assert_eq!(package.load_comments().unwrap().len(), 1);
    }
}
