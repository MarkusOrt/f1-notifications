CREATE TABLE IF NOT EXISTS "main"."audit_log" (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
  "for_weekend" INTEGER NOT NULL,
  "description" TEXT NOT NULL,
  "edited_by" TEXT NOT NULL,
  "edited_at" TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', CURRENT_TIMESTAMP)),
  "user_comment" TEXT,
  FOREIGN KEY ("for_weekend") REFERENCES "weekends" ("id")
);
