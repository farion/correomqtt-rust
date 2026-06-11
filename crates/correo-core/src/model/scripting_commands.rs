use crate::AppCommand;

use super::AppModel;

impl AppModel {
    pub(super) fn apply_scripting_command(&mut self, command: &AppCommand) -> bool {
        match command {
            AppCommand::SearchScripts(filter) => self.search_scripts(filter.clone()),
            AppCommand::SelectScriptConnection(connection_id) => {
                self.select_script_connection(connection_id);
            }
            AppCommand::SelectScript(name) => self.select_script(name.clone()),
            AppCommand::RequestCreateScript => self.request_create_script(),
            AppCommand::UpdateNewScriptName(name) => self.update_new_script_name(name.clone()),
            AppCommand::CancelCreateScript => self.cancel_create_script(),
            AppCommand::CreateScript => self.create_script(),
            AppCommand::UpdateScriptSource(source) => self.update_script_source(source.clone()),
            AppCommand::SaveScript => self.save_script(),
            AppCommand::DiscardScriptChanges => self.discard_script_changes(),
            AppCommand::RequestRenameScript => self.request_rename_script(),
            AppCommand::UpdateRenameScriptName(name) => {
                self.update_rename_script_name(name.clone());
            }
            AppCommand::CancelRenameScript => self.cancel_rename_script(),
            AppCommand::ConfirmRenameScript => self.confirm_rename_script(),
            AppCommand::RequestDeleteScript => self.request_delete_script(),
            AppCommand::CancelDeleteScript => self.cancel_delete_script(),
            AppCommand::ConfirmDeleteScript => self.confirm_delete_script(),
            AppCommand::SelectScriptDetailTab(tab) => self.select_script_detail_tab(*tab),
            AppCommand::SelectScriptExecution(execution_id) => {
                self.select_script_execution(execution_id.clone());
            }
            AppCommand::RunScript => self.run_script(),
            AppCommand::CancelScript => self.cancel_script(),
            AppCommand::ClearFinishedScriptExecutions => self.clear_finished_script_executions(),
            _ => return false,
        }
        true
    }
}
