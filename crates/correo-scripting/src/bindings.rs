use std::sync::Arc;

use rquickjs::{Ctx, Function, Object};

use crate::{client_bindings::build_client_factory, executor::HostState, ScriptLogLevel};

pub(crate) fn install_bindings<'js>(ctx: Ctx<'js>, state: Arc<HostState>) -> rquickjs::Result<()> {
    let globals = ctx.globals();
    globals.set("logger", build_logger(ctx.clone(), state.clone())?)?;

    let sleep_state = state.clone();
    globals.set(
        "sleep",
        Function::new(ctx.clone(), move |millis: i32| {
            sleep_state
                .sleep(millis)
                .map_err(|error| sleep_state.throw_host_error(error))
        })?
        .with_name("sleep")?,
    )?;

    let join_state = state.clone();
    globals.set(
        "join",
        Function::new(ctx.clone(), move || {
            join_state
                .check_cancelled()
                .map_err(|error| join_state.throw_host_error(error))
        })?
        .with_name("join")?,
    )?;

    globals.set("queue", build_queue(ctx.clone(), state.clone())?)?;
    globals.set("clientFactory", build_client_factory(ctx, state)?)?;
    Ok(())
}

fn build_logger<'js>(ctx: Ctx<'js>, state: Arc<HostState>) -> rquickjs::Result<Object<'js>> {
    let logger = Object::new(ctx.clone())?;
    let debug_state = state.clone();
    logger.set(
        "debug",
        Function::new(ctx.clone(), move |message: String| {
            debug_state
                .log(ScriptLogLevel::Debug, message)
                .map_err(|error| debug_state.throw_host_error(error))
        })?
        .with_name("logger.debug")?,
    )?;

    let info_state = state.clone();
    logger.set(
        "info",
        Function::new(ctx.clone(), move |message: String| {
            info_state
                .log(ScriptLogLevel::Info, message)
                .map_err(|error| info_state.throw_host_error(error))
        })?
        .with_name("logger.info")?,
    )?;

    let warn_state = state.clone();
    logger.set(
        "warn",
        Function::new(ctx.clone(), move |message: String| {
            warn_state
                .log(ScriptLogLevel::Warning, message)
                .map_err(|error| warn_state.throw_host_error(error))
        })?
        .with_name("logger.warn")?,
    )?;

    let error_state = state;
    logger.set(
        "error",
        Function::new(ctx, move |message: String| {
            error_state
                .log(ScriptLogLevel::Error, message)
                .map_err(|error| error_state.throw_host_error(error))
        })?
        .with_name("logger.error")?,
    )?;

    Ok(logger)
}

fn build_queue<'js>(ctx: Ctx<'js>, state: Arc<HostState>) -> rquickjs::Result<Object<'js>> {
    let queue = Object::new(ctx.clone())?;
    let process_state = state.clone();
    queue.set(
        "process",
        Function::new(ctx.clone(), move || {
            process_state
                .queue_process()
                .map_err(|error| process_state.throw_host_error(error))
        })?
        .with_name("queue.process")?,
    )?;

    let jump_out_state = state;
    queue.set(
        "jumpOut",
        Function::new(ctx, move || {
            jump_out_state
                .queue_jump_out()
                .map_err(|error| jump_out_state.throw_host_error(error))
        })?
        .with_name("queue.jumpOut")?,
    )?;
    Ok(queue)
}
