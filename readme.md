# F1 Notifications Bot

A self-hosted Discord bot that sends automated notifications for F1, F2, F3 
and F1 Academy sessions. Includes a web-based management UI with Discord 
OAuth2 authentication, allowing authorised moderators to manage events and 
sessions without touching the codebase or database.

Built with Rust, fully dockerized, and designed for production deployment.

## Configuration

Settings for the application are done using environment variables.
If available the application will try to read a .env file.

Here is an example `.env` file.

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

### Docker Compose

Here is a `docker-compose.yaml` example:

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

### Database Migrations

Before running the application, the database needs to be created and the schema be created.
for this run `docker compose run discord-bot /app/migrate` to run database migrations.

### Calendar messages

By default the bot won't create calendar messages, to generate these run
`docker compose run discord-bot /app/reserve-calendar-space`

### Web Interface

Authorised user IDs are currently configured in 
[`/src/http/auth/mod.rs`](./src/http/auth/mod.rs). 
Configurable auth management via the web interface is planned for a future release

By default the app exposes port 8123 for the HTTP Interface.

## Tech stack

- **Rust** with Tokio for async runtime
- **Axum** for HTTP API and web frontend
- **libsql** (SQLite) for data storage
- **Discord OAuth2** for moderator authentication
- **Docker** for containerised deployment
- **Sentry** (optional) for error reporting and observability

## Screenshots

![Screenshot of Calendar](/assets/github/screenshot_1.png?raw=true)
![Screenshot of Webui](/assets/github/screenshot_3.png?raw=true)
![Screenshot of Session Edit](/assets/github/screenshot_2.png?raw=true)

## License

This application is licensed under the MIT License ([see License](./LICENSE))
