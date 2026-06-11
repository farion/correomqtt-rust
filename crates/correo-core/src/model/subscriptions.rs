use crate::{AppModel, Diagnostic, WorkflowFeedback};

impl AppModel {
    pub(super) fn request_unsubscribe_all(&mut self) {
        let count = self.snapshot.workbench.subscribe.subscriptions.len();
        if count <= 1 {
            self.snapshot.workbench.subscribe.feedback = Some(WorkflowFeedback::warning(
                "Unsubscribe all requires more than one subscription.",
            ));
            return;
        }
        self.snapshot.workbench.subscribe.subscriptions.clear();
        self.snapshot.workbench.subscribe.feedback = Some(WorkflowFeedback::info(format!(
            "Unsubscribe all queued for {count} subscriptions."
        )));
        self.push_diagnostic(Diagnostic::info(format!(
            "Unsubscribe all command queued for {count} subscriptions."
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

    pub(super) fn set_subscription_messages_visible(&mut self, topic_filter: &str, visible: bool) {
        if let Some(subscription) = self
            .snapshot
            .workbench
            .subscribe
            .subscriptions
            .iter_mut()
            .find(|subscription| subscription.topic_filter == topic_filter)
        {
            subscription.messages_visible = visible;
        }
    }

    pub(super) fn set_all_subscription_messages_visible(&mut self, visible: bool) {
        for subscription in &mut self.snapshot.workbench.subscribe.subscriptions {
            subscription.messages_visible = visible;
        }
    }

    pub(super) fn select_subscription(&mut self, topic_filter: &str, extend: bool, toggle: bool) {
        let subscriptions = &mut self.snapshot.workbench.subscribe.subscriptions;
        let Some(index) = subscriptions
            .iter()
            .position(|subscription| subscription.topic_filter == topic_filter)
        else {
            return;
        };
        if extend {
            let anchor = subscriptions
                .iter()
                .rposition(|subscription| subscription.selected)
                .unwrap_or(index);
            let start = anchor.min(index);
            let end = anchor.max(index);
            for (row_index, subscription) in subscriptions.iter_mut().enumerate() {
                subscription.selected = (start..=end).contains(&row_index);
            }
        } else if toggle {
            subscriptions[index].selected = !subscriptions[index].selected;
        } else {
            for subscription in subscriptions.iter_mut() {
                subscription.selected = false;
            }
            subscriptions[index].selected = true;
        }
    }
}
