# F1 Notifications Bot

Creates notifications for F1 / F2 / F3 / F1Academy sessions.

Insert new Events and sessions using a nice frontend.
All times are converted from your local time into UTC (+00:00).

## Examples

![Screenshot of Calendar](/assets/github/screenshot_1.png?raw=true)
![Screenshot of Webui](/assets/github/screenshot_3.png?raw=true)
![Screenshot of Session Edit](/assets/github/screenshot_2.png?raw=true)

## Configuration

Settings for the application are done using environment variables.
if available the application will try to read a .env file.

Here is a example `.env` file.

```env
SENTRY_DSN=<Optional Sentry DSN for sentry error reporting>
PUBLIC_KEY=<Discord Application Public Key>
BOT_TOKEN=<Discord Bot Token>
CALENDAR_CHANNEL=<Channel Id for calendar posting>
F1_CHANNEL=<Channel Id for F1 Notifications>
FEEDER_CHANNEL=<Channel Id for F1 Feeder (F2/F3/F1Academy) Notifications>
CLIENT_ID=<Discord App Client Id>
CLIENT_SECRET=<Discord App Client Secret>
```

## Setup

You can easily run this Bot with Docker using the above configuration .env file.

here is a `docker-compose.yaml` example:

```yaml
services:
  discord-bot:
    container_name: f1-notif-bot
    restart: unless-stopped
    image: "codeberg.org/mto/f1-notifications:latest"
    stop_grace_period: 30s
    env_file: ./.env
    ports:
      - 127.0.0.1:8123:8123
    volumes:
      - ./db/:/app/database
```

For webui Access you need to change the list of allowed ids in the 
[`/src/http/auth/mod.rs`](./src/http/auth/mod.rs) file. This may change in the future.

By default the app exposes port 8123 for the HTTP Interface.


## Tech stack

- Rust with Tokio (async)
- Libsql (Sqlite) for Data storage
- Axum for Http Api / fronted

