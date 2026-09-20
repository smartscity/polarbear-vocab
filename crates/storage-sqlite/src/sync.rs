use std::path::Path;

use chrono::{SecondsFormat, Utc};
use polarbear_vocab_application::{ApplicationError, BackupRepository, SyncRepository};
use polarbear_vocab_domain::{SyncExportResultDto, SyncImportResultDto, SyncStatusDto};
use rusqlite::Connection;
use uuid::Uuid;

use crate::sync_models::{ApplyCounts, SyncCursor, SyncManifest};
use crate::{
    SqliteStore, backup_versions, database_error, sync_archive, sync_content, sync_peer, sync_user,
};

impl SyncRepository for SqliteStore {
    fn sync_status(&self) -> Result<SyncStatusDto, ApplicationError> {
        let (device_id, peers, base, last_sync_at) = {
            let mut user = self.user()?;
            (
                sync_peer::device_id(&mut user)?,
                sync_peer::peer_count(&user)?,
                sync_peer::base_cursors(&user)?,
                sync_peer::last_sync_at(&user)?,
            )
        };
        let content_pending = {
            let content = self.content()?;
            sync_content::pending_count(&content, base.content).map_err(database_error)?
        };
        let user_pending = {
            let user = self.user()?;
            sync_user::pending_count(&user, base.user).map_err(database_error)?
        };
        Ok(SyncStatusDto {
            device_id,
            peer_count: peers,
            pending_change_count: content_pending + user_pending,
            last_sync_at,
        })
    }

    fn export_sync(
        &self,
        path: &str,
        app_version: &str,
    ) -> Result<SyncExportResultDto, ApplicationError> {
        let (device_id, base, acknowledgements, user, user_cursor) = {
            let mut connection = self.user()?;
            let device_id = sync_peer::device_id(&mut connection)?;
            let base = sync_peer::base_cursors(&connection)?;
            let acknowledgements = sync_peer::acknowledgements(&connection)?;
            let (user, cursor) = sync_user::export(&connection, base.user)?;
            (device_id, base, acknowledgements, user, cursor)
        };
        let (content, content_cursor) = {
            let connection = self.content()?;
            sync_content::export(&connection, base.content)?
        };
        let content_bytes = serde_json::to_vec(&content).map_err(json_error)?;
        let user_bytes = serde_json::to_vec(&user).map_err(json_error)?;
        let package_id = Uuid::new_v4().to_string();
        let manifest = SyncManifest {
            format: "polarbear-vocab-sync".to_owned(),
            schema_version: 1,
            app_version: app_version.to_owned(),
            package_id: package_id.clone(),
            source_device_id: device_id,
            created_at: Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
            content_cursor,
            user_cursor,
            acknowledgements,
            content_sha256: sync_archive::digest(&content_bytes),
            user_sha256: sync_archive::digest(&user_bytes),
        };
        sync_archive::write(Path::new(path), &manifest, &content_bytes, &user_bytes)?;
        Ok(SyncExportResultDto {
            package_id,
            dataset_count: content.datasets.len() as u32,
            article_count: user.articles.len() as u32,
            review_event_count: user.review_events.len() as u32,
        })
    }

    fn import_sync(
        &self,
        path: &str,
        app_version: &str,
    ) -> Result<SyncImportResultDto, ApplicationError> {
        let package = sync_archive::read(Path::new(path))?;
        let (device_id, already_applied) = {
            let mut user = self.user()?;
            let device_id = sync_peer::device_id(&mut user)?;
            let applied = sync_peer::package_applied(&user, &package.manifest.package_id)?;
            (device_id, applied)
        };
        if package.manifest.source_device_id == device_id {
            return Err(ApplicationError::InvalidInput(
                "cannot import a sync package created by this device".to_owned(),
            ));
        }
        if already_applied {
            return Ok(SyncImportResultDto {
                already_applied: true,
                dataset_count: 0,
                article_count: 0,
                review_event_count: 0,
                conflict_count: 0,
            });
        }
        let acknowledged = self.clamped_acknowledgement(&package.manifest, &device_id)?;
        let safety = backup_versions::new_path(&self.paths.user, "pre-sync")?;
        self.export_backup(&safety.to_string_lossy(), app_version)?;
        backup_versions::prune(&self.paths.user)?;
        let applied = self.apply_package(&package, acknowledged);
        match applied {
            Ok(counts) => Ok(import_result(counts)),
            Err(error) => {
                self.restore_backup_archive(&safety).map_err(|rollback| {
                    ApplicationError::Infrastructure(format!(
                        "sync import failed ({error}); rollback failed ({rollback}); backup is {}",
                        safety.display()
                    ))
                })?;
                Err(error)
            }
        }
    }
}

impl SqliteStore {
    fn apply_package(
        &self,
        package: &sync_archive::SyncPackage,
        acknowledged: SyncCursor,
    ) -> Result<ApplyCounts, ApplicationError> {
        let mut content = self.content()?;
        let mut counts = sync_content::apply(
            &mut content,
            &package.content,
            acknowledged.content,
            &package.manifest.source_device_id,
            &package.manifest.package_id,
        )?;
        drop(content);
        let user_counts = {
            let mut user = self.user()?;
            sync_user::apply(&mut user, &package.user, &package.manifest, acknowledged)?
        };
        counts.articles = user_counts.articles;
        counts.review_events = user_counts.review_events;
        counts.conflicts += user_counts.conflicts;
        Ok(counts)
    }

    fn clamped_acknowledgement(
        &self,
        manifest: &SyncManifest,
        device_id: &str,
    ) -> Result<SyncCursor, ApplicationError> {
        let requested = sync_peer::local_acknowledged_by_source(manifest, device_id);
        let content_max = {
            let content = self.content()?;
            maximum_cursor(&content)?
        };
        let user_max = {
            let user = self.user()?;
            maximum_cursor(&user)?
        };
        Ok(SyncCursor {
            content: requested.content.clamp(0, content_max),
            user: requested.user.clamp(0, user_max),
        })
    }
}

fn maximum_cursor(connection: &Connection) -> Result<i64, ApplicationError> {
    connection
        .query_row("SELECT COALESCE(MAX(seq), 0) FROM sync_change", [], |row| {
            row.get(0)
        })
        .map_err(database_error)
}

fn import_result(counts: ApplyCounts) -> SyncImportResultDto {
    SyncImportResultDto {
        already_applied: false,
        dataset_count: counts.datasets,
        article_count: counts.articles,
        review_event_count: counts.review_events,
        conflict_count: counts.conflicts,
    }
}

fn json_error(error: serde_json::Error) -> ApplicationError {
    ApplicationError::Infrastructure(format!("cannot serialize sync package: {error}"))
}
