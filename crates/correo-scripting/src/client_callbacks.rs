use std::{cell::RefCell, rc::Rc};

use rquickjs::{Function, Value};

pub(crate) type MessageSubscriptions<'js> = Rc<RefCell<Vec<MessageSubscription<'js>>>>;

pub(crate) struct MessageSubscription<'js> {
    topic_filter: String,
    callback: Function<'js>,
    kind: CallbackKind,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CallbackKind {
    Message,
    IncomingTransform,
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
        kind: CallbackKind::Message,
    });
}

pub(crate) fn register_incoming_transform<'js>(
    subscriptions: &MessageSubscriptions<'js>,
    topic_filter: String,
    callback: Function<'js>,
) {
    subscriptions.borrow_mut().push(MessageSubscription {
        topic_filter,
        callback,
        kind: CallbackKind::IncomingTransform,
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
    let transforms = matching_callbacks(subscriptions, topic, CallbackKind::IncomingTransform);
    let payload = apply_incoming_transforms(transforms, payload)?;
    let callbacks = matching_callbacks(subscriptions, topic, CallbackKind::Message);

    for callback in callbacks {
        callback.call::<_, ()>((payload.clone(),))?;
    }
    Ok(())
}

fn matching_callbacks<'js>(
    subscriptions: &MessageSubscriptions<'js>,
    topic: &str,
    kind: CallbackKind,
) -> Vec<Function<'js>> {
    subscriptions
        .borrow()
        .iter()
        .filter(|subscription| {
            subscription.kind == kind && topic_matches(subscription.topic_filter.as_str(), topic)
        })
        .map(|subscription| subscription.callback.clone())
        .collect()
}

fn apply_incoming_transforms<'js>(
    transforms: Vec<Function<'js>>,
    payload: &str,
) -> rquickjs::Result<String> {
    let mut payload = payload.to_owned();
    for transform in transforms {
        let value = transform.call::<_, Value>((payload.clone(),))?;
        if !value.is_undefined() && !value.is_null() {
            payload = value_string(value)?;
        }
    }
    Ok(payload)
}

fn value_string(value: Value<'_>) -> rquickjs::Result<String> {
    if value.is_string() {
        value.get::<String>()
    } else if let Some(number) = value.as_number() {
        Ok(number.to_string())
    } else if let Some(boolean) = value.as_bool() {
        Ok(boolean.to_string())
    } else {
        Ok("[object Object]".to_owned())
    }
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
