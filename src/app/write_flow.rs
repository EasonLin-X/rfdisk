use std::io::Write;

use crate::{
    app::state::App,
    backend::write::{write_partition_table, WriteDiskResult},
    log::writer::open_write_log,
    model::RiskLevel,
};

impl App {
    pub(crate) fn write_all_committed_drafts(&mut self) -> Result<String, String> {
        let (log_path, mut log) =
            open_write_log().map_err(|err| format!("Cannot open log: {err}"))?;

        let mut successes = Vec::new();
        let mut reload_failures = Vec::new();
        let mut failures = Vec::new();
        let draft_ids: Vec<String> = self.drafts.keys().cloned().collect();
        let preview = self.write_preview();

        writeln!(log, "rfdisk write started").ok();
        writeln!(log, "preview summary: {}", preview.status_summary()).ok();
        for disk_id in draft_ids {
            let Some(draft) = self.drafts.get(&disk_id).cloned() else {
                continue;
            };
            let Some(disk) = self
                .disks
                .iter()
                .find(|disk| disk.stable_id() == disk_id)
                .cloned()
            else {
                failures.push(format!("{disk_id}: disk disappeared"));
                continue;
            };

            writeln!(log, "\n== {} ==", disk.dev_path).ok();
            if let Some(disk_preview) = preview
                .disks
                .iter()
                .find(|disk_preview| disk_preview.disk_path == disk.dev_path)
            {
                writeln!(log, "{}", disk_preview.log_summary()).ok();
            }

            if let Some(blocked_preview) = preview.disks.iter().find(|disk_preview| {
                disk_preview.disk_path == disk.dev_path && disk_preview.risk == RiskLevel::Blocked
            }) {
                let msg = blocked_preview.status_summary();
                writeln!(log, "SKIP: {msg}").ok();
                failures.push(msg);
                continue;
            }

            if disk.is_protected() {
                let msg = format!("{} {}", disk.dev_path, disk.guard_message());
                writeln!(log, "SKIP: {msg}").ok();
                failures.push(msg);
                continue;
            }

            match write_partition_table(&disk, &draft, &mut log) {
                WriteDiskResult::WrittenAndReloaded => {
                    successes.push(disk.dev_path.clone());
                    self.drafts.remove(&disk_id);
                }
                WriteDiskResult::WrittenButReloadFailed { message } => {
                    let msg = format!("{}: {message}", disk.dev_path);
                    writeln!(log, "PARTIAL: {msg}").ok();
                    reload_failures.push(msg);
                }
                WriteDiskResult::WriteFailed { message } => {
                    let msg = format!("{}: {message}", disk.dev_path);
                    writeln!(log, "FAILED: {msg}").ok();
                    failures.push(msg);
                }
            }
        }

        self.refresh(false);

        if failures.is_empty() && reload_failures.is_empty() {
            Ok(format!(
                "Write complete for {} disk(s). Log: {}",
                successes.len(),
                log_path.display()
            ))
        } else if failures.is_empty() {
            Err(format!(
                "Write partially complete: partition table was written, but kernel did not reload it for {} disk(s). First: {}. Log: {}",
                reload_failures.len(),
                reload_failures[0],
                log_path.display()
            ))
        } else {
            Err(format!(
                "Write finished with {} failure(s). First: {}. Log: {}",
                failures.len(),
                failures[0],
                log_path.display()
            ))
        }
    }
}
