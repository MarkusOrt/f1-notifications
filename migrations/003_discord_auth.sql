CREATE TABLE IF NOT EXISTS "main"."users" (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
  "name" TEXT NOT NULL,
  "discord_id" TEXT NOT NULL,
  "discord_token" TEXT NOT NULL,
  "created_at" TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', CURRENT_TIMESTAMP))
);

create table if not exists "main"."user_sessions" (
  "id" integer primary key autoincrement not null,
  "user_id" integer not null,
  "token" text not null,
  "expires_at" TEXT NOT NULL,
  "created_at" text not null default (strftime('%y-%m-%dt%h:%m:%sz', current_timestamp)),
  foreign key ("user_id") references "users" ("id")
);

create table if not exists "main"."auth_sessions" (
  "id" integer primary key autoincrement not null,
  "token" text not null,
  "expires_at" TEXT NOT NULL,
  "created_at" text not null default (strftime('%y-%m-%dt%h:%m:%sz', current_timestamp))
);

