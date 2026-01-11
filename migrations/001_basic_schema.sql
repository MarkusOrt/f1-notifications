CREATE TABLE IF NOT EXISTS "main"."weekends" (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
  "name" TEXT NOT NULL,
  "series" TEXT NOT NULL,
  "icon" TEXT NOT NULL DEFAULT 'F1',
  "year" INTEGER NOT NULL DEFAULT (strftime('%Y', CURRENT_TIMESTAMP)),
  "start_date" TEXT NOT NULL DEFAULT (strftime('%Y-%m-%d', CURRENT_TIMESTAMP)),
  "status" TEXT NOT NULL,
  "created_at" TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', CURRENT_TIMESTAMP))
);

CREATE TABLE IF NOT EXISTS "main"."sessions" (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
  "weekend_id" INTEGER NOT NULL,
  "start_time" TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', CURRENT_TIMESTAMP)),
  "name" TEXT NOT NULL,
  "duration" INTEGER NOT NULL DEFAULT '3600',
  "notify" TEXT NOT NULL,
  "status" TEXT NOT NULL,
  "created_at" TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', CURRENT_TIMESTAMP)),
  FOREIGN KEY ("weekend_id") REFERENCES "weekends" ("id")
);

CREATE TABLE IF NOT EXISTS "main"."messages" (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
  "discord_id" TEXT NOT NULL,
  "discord_channel" TEXT NOT NULL,
  "kind" TEXT NOT NULL,
  "series" TEXT NOT NULL,
  "hash" TEXT NOT NULL DEFAULT '0',
  "expires_at" TEXT NOT NULL,
  "created_at" TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', CURRENT_TIMESTAMP))
);
