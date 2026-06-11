use std::collections::{HashMap, HashSet};

use crate::ScriptFeedback;

use super::AppModel;

impl AppModel {
    pub(super) fn clear_finished_script_executions(&mut self) {
        let finished_ids = self
            .snapshot
            .scripts
            .executions
            .iter()
            .filter(|execution| execution.status.is_terminal())
            .map(|execution| execution.execution_id.clone())
            .collect::<HashSet<_>>();

        if finished_ids.is_empty() {
            self.set_script_feedback(ScriptFeedback::info("No finished execution logs to clear."));
            return;
        }

        let cleared = finished_ids.len();
        let scripts = &mut self.snapshot.scripts;
        scripts
            .executions
            .retain(|execution| !finished_ids.contains(&execution.execution_id));
        scripts
            .log_lines
            .retain(|line| !finished_ids.contains(&line.execution_id));

        if scripts
            .selected_execution_id
            .as_ref()
            .is_some_and(|id| finished_ids.contains(id))
        {
            scripts.selected_execution_id = scripts.active_execution_id.clone().or_else(|| {
                scripts
                    .executions
                    .first()
                    .map(|execution| execution.execution_id.clone())
            });
        }

        let mut counts = HashMap::new();
        for execution in &scripts.executions {
            *counts.entry(execution.script_name.clone()).or_insert(0) += 1;
        }
        for script in &mut scripts.scripts {
            script.execution_count = counts.get(&script.name).copied().unwrap_or(0);
        }

        if scripts
            .executions
            .iter()
            .all(|execution| execution.error.is_none())
        {
            scripts.last_error = None;
        }
        self.set_script_feedback(ScriptFeedback::info(format!(
            "Cleared {cleared} finished execution log(s)."
        )));
    }
}
