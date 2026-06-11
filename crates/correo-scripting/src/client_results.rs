use std::sync::Arc;

use rquickjs::{prelude::Opt, Ctx, Function};

use crate::{
    client_args::MqttOperation,
    client_callbacks::{dispatch_message_callbacks, MessageSubscriptions},
    executor::HostState,
    ScriptingError, ScriptingResult,
};

pub(crate) fn finish_async_result<'js>(
    state: &Arc<HostState>,
    result: ScriptingResult<()>,
    on_success: Option<Function<'js>>,
    on_error: Option<Function<'js>>,
    publish: Option<(&MqttOperation, &MessageSubscriptions<'js>)>,
) -> rquickjs::Result<()> {
    match result {
        Ok(()) => {
            if let Some((operation, subscriptions)) = publish {
                dispatch_published_message(operation, subscriptions)?;
            }
            call_optional_callback(on_success)
        }
        Err(ScriptingError::Cancelled) => Err(state.throw_host_error(ScriptingError::Cancelled)),
        Err(_) if on_error.is_none() => Ok(()),
        Err(_) => call_optional_callback(on_error),
    }
}

pub(crate) fn finish_publish_result<'js>(
    state: &Arc<HostState>,
    result: ScriptingResult<()>,
    operation: &MqttOperation,
    subscriptions: &MessageSubscriptions<'js>,
) -> rquickjs::Result<()> {
    match result {
        Ok(()) => dispatch_published_message(operation, subscriptions),
        Err(error) => Err(state.throw_host_error(error)),
    }
}

pub(crate) fn promise_adapter<'js>(
    ctx: Ctx<'js>,
    state: Arc<HostState>,
    operation: MqttOperation,
    name: &'static str,
    subscriptions: MessageSubscriptions<'js>,
) -> rquickjs::Result<Function<'js>> {
    Function::new(
        ctx,
        move |resolve: Opt<Function>, reject: Opt<Function>| match operation.run(&state) {
            Ok(()) => {
                dispatch_published_message(&operation, &subscriptions)?;
                call_required_callback(&state, resolve.0, "promise resolve callback")
            }
            Err(error) => match reject.0 {
                Some(reject) => reject.call::<_, ()>(()),
                None => Err(state.throw_host_error(error)),
            },
        },
    )?
    .with_name(name)
}

fn dispatch_published_message<'js>(
    operation: &MqttOperation,
    subscriptions: &MessageSubscriptions<'js>,
) -> rquickjs::Result<()> {
    let Some((topic, payload)) = operation.published_message() else {
        return Ok(());
    };
    dispatch_message_callbacks(subscriptions, topic, payload)
}

fn call_optional_callback(callback: Option<Function<'_>>) -> rquickjs::Result<()> {
    if let Some(callback) = callback {
        callback.call::<_, ()>(())
    } else {
        Ok(())
    }
}

fn call_required_callback(
    state: &HostState,
    callback: Option<Function<'_>>,
    label: &str,
) -> rquickjs::Result<()> {
    callback
        .ok_or_else(|| {
            state.throw_host_error(ScriptingError::HostApi(format!("{label} is required")))
        })?
        .call::<_, ()>(())
}
