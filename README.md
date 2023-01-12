# Start and stop system (STT)

Start and stop system for applications to save your budget on hourly billing VPS.

## Service

A service consists of start/stop scripts, and a `busy` detection service.

### "Busy" detection

STT calls this function repeatedly (interval can be configurated), and execute corresponding scripts according to its state.

When a service is being used (aka busy), does nothing.

When a service is not being used, STT will schedule a double-check (this timeout can also be configurated) and then call the stop script.

### Wrapper

Occasionally you want to wrap a TCP service, e.g. A Minecraft server. In this case, STT can wrap the TCP socket.

If service is not running currently, it returns an informational message to let user wait for server start and then call the start script.
