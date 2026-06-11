use std::{cell::RefCell, rc::Rc};

use rquickjs::Function;

pub(crate) type MessageSubscriptions<'js> = Rc<RefCell<Vec<MessageSubscription<'js>>>>;

pub(crate) struct MessageSubscription<'js> {
    topic_filter: String,
    callback: Function<'js>,
}

pub(crate) fn new_message_subscriptions<'js>() -> MessageSubscriptions<'js> {
    Rc::new(RefCell::new(Vec::new()))
}

pub(crate) fn register_message_callback<'js>(
    subscriptions: &MessageSubscriptions<'js>,
    topic_filter: String,
    callback: Option<Function<'js>>,
) {
    let Some(callback) = callback else {
        return;
    };
    subscriptions.borrow_mut().push(MessageSubscription {
        topic_filter,
        callback,
    });
}

pub(crate) fn remove_message_callbacks(
    subscriptions: &MessageSubscriptions<'_>,
    topic_filter: &str,
) {
    subscriptions
        .borrow_mut()
        .retain(|subscription| subscription.topic_filter != topic_filter);
}

pub(crate) fn clear_message_callbacks(subscriptions: &MessageSubscriptions<'_>) {
    subscriptions.borrow_mut().clear();
}

pub(crate) fn dispatch_message_callbacks<'js>(
    subscriptions: &MessageSubscriptions<'js>,
    topic: &str,
    payload: &str,
) -> rquickjs::Result<()> {
    let callbacks = subscriptions
        .borrow()
        .iter()
        .filter(|subscription| topic_matches(subscription.topic_filter.as_str(), topic))
        .map(|subscription| subscription.callback.clone())
        .collect::<Vec<_>>();

    for callback in callbacks {
        callback.call::<_, ()>((payload.to_owned(),))?;
    }
    Ok(())
}

fn topic_matches(filter: &str, topic: &str) -> bool {
    let filter_levels = filter.split('/').collect::<Vec<_>>();
    let topic_levels = topic.split('/').collect::<Vec<_>>();
    let mut index = 0;
    while index < filter_levels.len() {
        let filter_level = filter_levels[index];
        if filter_level == "#" {
            return index + 1 == filter_levels.len();
        }
        if index >= topic_levels.len() {
            return false;
        }
        if filter_level != "+" && filter_level != topic_levels[index] {
            return false;
        }
        index += 1;
    }
    index == topic_levels.len()
}
