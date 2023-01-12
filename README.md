# Start and stop system (STT)

Start and stop system for applications to save your budget on hourly billing VPS.

## Service

A service consists of start/stop scripts, and a `busy` detection service.

### "Busy" detection

STT calls this function repeatedly (interval can be configurated), and execute corresponding scripts according to its state.

When a service is being used (aka busy), does nothing.

When a service is not being used, STT will schedule a double-check (this timeout can also be configurated) and then call the stop script.

### Wrapper

Occasionally you want to wrap a TCP service, e.g. A Minecraft server.

In this case, If Minecraft server is not running currently, STT occupies your server port, e.g. 25565 at first. Any user that tries to connect will receive an informational message to let user wait for server start. After that, STT will release the occupation of this port and then call the start script. This way you don't have to deal with the annoying TCP packet redirection.

STT can also be configurated to automatically check your service's running status and reoccupy the port, this usually helps if your service fail to start.

## TODO

- [ ] Keep-Alive
