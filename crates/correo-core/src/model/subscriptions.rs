use crate::{AppModel, Diagnostic, WorkflowFeedback};

impl AppModel {
    pub(super) fn request_unsubscribe_all(&mut self) {
        let count = self.active_subscription_count();
        if count <= 1 {
            self.snapshot.workbench.subscribe.feedback = Some(WorkflowFeedback::warning(
                "Unsubscribe all requires more than one active subscription.",
            ));
            return;
        }
        self.snapshot
            .workbench
            .subscribe
            .unsubscribe_all_confirmation_count = Some(count);
        self.snapshot.workbench.subscribe.feedback = Some(WorkflowFeedback::warning(format!(
            "Confirm unsubscribe from {count} active subscriptions."
        )));
    }

    pub(super) fn cancel_unsubscribe_all(&mut self) {
        if let Some(count) = self
            .snapshot
            .workbench
            .subscribe
            .unsubscribe_all_confirmation_count
            .take()
        {
            self.snapshot.workbench.subscribe.feedback = Some(WorkflowFeedback::info(format!(
                "Kept {count} active subscriptions."
            )));
        }
    }

    pub(super) fn confirm_unsubscribe_all(&mut self) {
        let Some(count) = self
            .snapshot
            .workbench
            .subscribe
            .unsubscribe_all_confirmation_count
            .take()
        else {
            return;
        };
        self.snapshot.workbench.subscribe.feedback = Some(WorkflowFeedback::info(format!(
            "Unsubscribe all queued for {count} active subscriptions."
        )));
        self.push_diagnostic(Diagnostic::info(format!(
            "Unsubscribe all command queued for {count} active subscriptions."
        )));
    }

    fn active_subscription_count(&self) -> usize {
        self.snapshot
            .workbench
            .subscribe
            .subscriptions
            .iter()
            .filter(|subscription| subscription.active)
            .count()
    }
}
