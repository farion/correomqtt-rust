use std::sync::Arc;

use rquickjs::{
    prelude::{Opt, Rest},
    Ctx, Function, Object, Value,
};

use crate::{
    client_args::{
        parse_async_publish_args, parse_async_subscribe_args, parse_async_unsubscribe_args,
        parse_publish_args, parse_subscribe_args, parse_unsubscribe_args, MqttOperation,
    },
    executor::HostState,
    ScriptingError, ScriptingResult,
};

pub(crate) fn build_client_factory<'js>(
    ctx: Ctx<'js>,
    state: Arc<HostState>,
) -> rquickjs::Result<Object<'js>> {
    let factory = Object::new(ctx.clone())?;
    factory.set(
        "getBlockingClient",
        function_creating_client(
            ctx.clone(),
            "getBlockingClient",
            state.clone(),
            ClientMode::Blocking,
        )?,
    )?;
    factory.set(
        "getAsyncClient",
        function_creating_client(
            ctx.clone(),
            "getAsyncClient",
            state.clone(),
            ClientMode::Async,
        )?,
    )?;
    factory.set(
        "getPromiseClient",
        function_creating_client(ctx, "getPromiseClient", state, ClientMode::Promise)?,
    )?;
    Ok(factory)
}

#[derive(Clone, Copy)]
enum ClientMode {
    Blocking,
    Async,
    Promise,
}

fn build_client<'js>(
    ctx: Ctx<'js>,
    state: Arc<HostState>,
    mode: ClientMode,
) -> rquickjs::Result<Object<'js>> {
    let client = Object::new(ctx.clone())?;
    install_publish(ctx.clone(), &client, state.clone(), mode)?;
    install_subscribe(ctx.clone(), &client, state.clone(), mode)?;
    install_unsubscribe(ctx, &client, state, mode)?;
    Ok(client)
}

fn function_creating_client<'js>(
    ctx: Ctx<'js>,
    name: &'static str,
    state: Arc<HostState>,
    mode: ClientMode,
) -> rquickjs::Result<Function<'js>> {
    Function::new(ctx, move |ctx: Ctx<'js>| {
        build_client(ctx, state.clone(), mode)
    })?
    .with_name(name)
}

fn install_publish<'js>(
    ctx: Ctx<'js>,
    client: &Object<'js>,
    state: Arc<HostState>,
    mode: ClientMode,
) -> rquickjs::Result<()> {
    match mode {
        ClientMode::Blocking => {
            let publish_state = state;
            client.set(
                "publish",
                Function::new(ctx, move |Rest(args): Rest<Value<'js>>| {
                    let result = parse_publish_args(&args)
                        .map(MqttOperation::Publish)
                        .and_then(|operation| operation.run(&publish_state));
                    result.map_err(|error| publish_state.throw_host_error(error))
                })?
                .with_name("publish")?,
            )
        }
        ClientMode::Async => {
            let publish_state = state;
            client.set(
                "publish",
                Function::new(ctx, move |Rest(args): Rest<Value<'js>>| {
                    let (publish, on_success, on_error) = parse_async_publish_args(args)
                        .map_err(|error| publish_state.throw_host_error(error))?;
                    finish_async_result(
                        &publish_state,
                        MqttOperation::Publish(publish).run(&publish_state),
                        on_success,
                        on_error,
                    )
                })?
                .with_name("publish")?,
            )
        }
        ClientMode::Promise => {
            let publish_state = state;
            client.set(
                "publish",
                Function::new(
                    ctx.clone(),
                    move |ctx: Ctx<'js>, Rest(args): Rest<Value<'js>>| {
                        let operation = parse_publish_args(&args)
                            .map(MqttOperation::Publish)
                            .map_err(|error| publish_state.throw_host_error(error))?;
                        promise_adapter(ctx, publish_state.clone(), operation, "publish")
                    },
                )?
                .with_name("publish")?,
            )
        }
    }
}

fn install_subscribe<'js>(
    ctx: Ctx<'js>,
    client: &Object<'js>,
    state: Arc<HostState>,
    mode: ClientMode,
) -> rquickjs::Result<()> {
    match mode {
        ClientMode::Blocking => {
            let subscribe_state = state;
            client.set(
                "subscribe",
                Function::new(ctx, move |Rest(args): Rest<Value<'js>>| {
                    let result = parse_subscribe_args(&args)
                        .map(MqttOperation::Subscribe)
                        .and_then(|operation| operation.run(&subscribe_state));
                    result.map_err(|error| subscribe_state.throw_host_error(error))
                })?
                .with_name("subscribe")?,
            )
        }
        ClientMode::Async => {
            let subscribe_state = state;
            client.set(
                "subscribe",
                Function::new(ctx, move |Rest(args): Rest<Value<'js>>| {
                    let (subscribe, on_success, on_error) = parse_async_subscribe_args(args)
                        .map_err(|error| subscribe_state.throw_host_error(error))?;
                    finish_async_result(
                        &subscribe_state,
                        MqttOperation::Subscribe(subscribe).run(&subscribe_state),
                        on_success,
                        on_error,
                    )
                })?
                .with_name("subscribe")?,
            )
        }
        ClientMode::Promise => {
            let subscribe_state = state;
            client.set(
                "subscribe",
                Function::new(
                    ctx.clone(),
                    move |ctx: Ctx<'js>, Rest(args): Rest<Value<'js>>| {
                        let operation = parse_subscribe_args(&args)
                            .map(MqttOperation::Subscribe)
                            .map_err(|error| subscribe_state.throw_host_error(error))?;
                        promise_adapter(ctx, subscribe_state.clone(), operation, "subscribe")
                    },
                )?
                .with_name("subscribe")?,
            )
        }
    }
}

fn install_unsubscribe<'js>(
    ctx: Ctx<'js>,
    client: &Object<'js>,
    state: Arc<HostState>,
    mode: ClientMode,
) -> rquickjs::Result<()> {
    match mode {
        ClientMode::Blocking => {
            let unsubscribe_state = state;
            client.set(
                "unsubscribe",
                Function::new(ctx, move |Rest(args): Rest<Value<'js>>| {
                    let result = parse_unsubscribe_args(&args)
                        .map(MqttOperation::Unsubscribe)
                        .and_then(|operation| operation.run(&unsubscribe_state));
                    result.map_err(|error| unsubscribe_state.throw_host_error(error))
                })?
                .with_name("unsubscribe")?,
            )
        }
        ClientMode::Async => {
            let unsubscribe_state = state;
            client.set(
                "unsubscribe",
                Function::new(ctx, move |Rest(args): Rest<Value<'js>>| {
                    let (topic_filter, on_success, on_error) =
                        parse_async_unsubscribe_args(args)
                            .map_err(|error| unsubscribe_state.throw_host_error(error))?;
                    finish_async_result(
                        &unsubscribe_state,
                        MqttOperation::Unsubscribe(topic_filter).run(&unsubscribe_state),
                        on_success,
                        on_error,
                    )
                })?
                .with_name("unsubscribe")?,
            )
        }
        ClientMode::Promise => {
            let unsubscribe_state = state;
            client.set(
                "unsubscribe",
                Function::new(
                    ctx.clone(),
                    move |ctx: Ctx<'js>, Rest(args): Rest<Value<'js>>| {
                        let operation = parse_unsubscribe_args(&args)
                            .map(MqttOperation::Unsubscribe)
                            .map_err(|error| unsubscribe_state.throw_host_error(error))?;
                        promise_adapter(ctx, unsubscribe_state.clone(), operation, "unsubscribe")
                    },
                )?
                .with_name("unsubscribe")?,
            )
        }
    }
}

fn finish_async_result<'js>(
    state: &Arc<HostState>,
    result: ScriptingResult<()>,
    on_success: Option<Function<'js>>,
    on_error: Option<Function<'js>>,
) -> rquickjs::Result<()> {
    match result {
        Ok(()) => call_optional_callback(on_success),
        Err(ScriptingError::Cancelled) => Err(state.throw_host_error(ScriptingError::Cancelled)),
        Err(_) if on_error.is_none() => Ok(()),
        Err(_) => call_optional_callback(on_error),
    }
}

fn promise_adapter<'js>(
    ctx: Ctx<'js>,
    state: Arc<HostState>,
    operation: MqttOperation,
    name: &'static str,
) -> rquickjs::Result<Function<'js>> {
    Function::new(
        ctx,
        move |resolve: Opt<Function>, reject: Opt<Function>| match operation.run(&state) {
            Ok(()) => call_required_callback(&state, resolve.0, "promise resolve callback"),
            Err(error) => match reject.0 {
                Some(reject) => reject.call::<_, ()>(()),
                None => Err(state.throw_host_error(error)),
            },
        },
    )?
    .with_name(name)
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
