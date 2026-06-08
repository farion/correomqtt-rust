logger.info("synthetic heartbeat publish");
clientFactory.getBlockingClient().publish("alerts/status", "ok");
