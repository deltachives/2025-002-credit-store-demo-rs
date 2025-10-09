-- Your SQL goes here
CREATE TABLE credit_store (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  person TEXT UNIQUE NOT NULL,
  credits INTEGER NOT NULL
);

CREATE TABLE credit_store_head (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  person TEXT UNIQUE NOT NULL,
  credits INTEGER NOT NULL
);

CREATE TABLE credit_store_version (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  opt_event_id INTEGER NULL REFERENCES credit_store_version(id)
);

CREATE TABLE credit_store_events (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  person TEXT UNIQUE NOT NULL,
  credits INTEGER NOT NULL,
  event_action TEXT CHECK(event_action IN ('insert', 'update', 'delete', 'frame')) NOT NULL,
  opt_object_id INTEGER NULL,
  opt_event_id INTEGER NULL REFERENCES credit_store_events(id),
  opt_event_arg INTEGER NULL,
  event_stack_level INTEGER NOT NULL,
  created_on TEXT NOT NULL
);