CREATE TABLE coin_store_diffs (
  id INTEGER NOT NULL PRIMARY KEY,
  obj_id INTEGER NOT NULL,
  person TEXT NOT NULL,
  coins INTEGER NOT NULL
);

CREATE TABLE coin_store_events (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  opt_diff_id INTEGER NULL REFERENCES coin_store_diffs(id),
  ev_action TEXT CHECK(ev_action IN ('insert', 'update', 'delete', 'open', 'close', 'reopen')) NOT NULL,
  span INTEGER NOT NULL,
  frame INTEGER NOT NULL,
  created_on_ts REAL NOT NULL,
  ev_desc TEXT NOT NULL
);

INSERT INTO coin_store_diffs (id, obj_id, person, coins) VALUES
  (1, 1, 'person001', 10),
  (2, 1, 'Z', 15),
  (3, 1, 'A', 5),
  (4, 2, 'person002', 7),
  (5, 2, 'person002', 14),
  (6, 3, 'person003', 8),
  (7, 1, 'C', 0)
;

INSERT INTO coin_store_events (opt_diff_id, ev_action, span, frame, created_on_ts, ev_desc) VALUES
  (null, 'open', 1, 1, 1758935304.168087602, 'New Years Eve'),
  (1, 'insert', 1, 1, 1758935465.865269566, ''),
  (2, 'update', 1, 1, 1758935520.052601247, ''),
  (3, 'update', 1, 1, 1758935543.867800435, ''),
  (null, 'open', 2, 1, unixepoch('subsec') + 0.1, 'New Years Eve Plan 1'),
  (4, 'insert', 2, 1, unixepoch('subsec') + 0.2, ''),
  (5, 'update', 2, 1, unixepoch('subsec') + 0.3, ''),
  (null, 'close', 2, 1, unixepoch('subsec') + 0.4, 'New Years Eve Plan 1'),
  (6, 'insert', 1, 1, unixepoch('subsec') + 0.5, ''),
  (7, 'delete', 1, 1, unixepoch('subsec') + 0.6, ''),
  (null, 'open', 2, 2, unixepoch('subsec') + 0.7, 'New Years Eve Plan 2'),
  (null, 'open', 1, 2, unixepoch('subsec') + 0.8, 'Birthday Party')
;

CREATE VIEW v_coin_store_events_grouped AS
  WITH RECURSIVE duplicator(dup, ev_id, obj_id, ev_action, span, frame, created_on_ts, person, coins, ev_desc) AS (
    SELECT 1, t1.id, t2.obj_id, t1.ev_action, t1.span, t1.frame, t1.created_on_ts, t2.person, t2.coins, t1.ev_desc
		FROM coin_store_events AS t1
		INNER JOIN coin_store_diffs AS t2
			ON t1.opt_diff_id = t2.id
		WHERE ev_action != 'open' AND ev_action != 'close' AND ev_action != 'reopen'
    UNION
    SELECT dup + 1, ev_id, obj_id, ev_action, span, frame, created_on_ts, person, coins, ev_desc
		FROM duplicator
		WHERE
			(dup + 1) <= (
				SELECT COUNT(*)
				FROM coin_store_events
				WHERE ev_action = 'open'
			)
  )
  SELECT t2.*, t1.*
	FROM duplicator AS t1
	JOIN
		(
			SELECT ROW_NUMBER() OVER () AS grp_id, *
			FROM (
				SELECT u1.span AS grp_span, u1.frame AS grp_frame, u1.created_on_ts AS grp_created_on_ts
				FROM coin_store_events AS u1
				WHERE ev_action = 'open'
				)
		) AS t2
	ON t1.dup = t2.grp_id
	WHERE
		(t1.frame == t2.grp_frame AND t1.span == t2.grp_span) OR
		(t1.span < t2.grp_span AND t1.created_on_ts < t2.grp_created_on_ts)
	ORDER BY
		t1.created_on_ts
  ;

CREATE VIEW v_coin_store_objects AS
  WITH
    aggr AS (
      SELECT
        grp_id, grp_span, grp_frame, obj_id,
        SUM(coins) AS coins
			FROM v_coin_store_events_grouped
			GROUP BY obj_id, grp_id
    ),
    latest AS (
      SELECT
				grp_id, obj_id, obj_state, person
			FROM (
				SELECT
					grp_id, obj_id, person,
					ev_action AS obj_state,
					ROW_NUMBER() OVER (PARTITION BY grp_id, obj_id ORDER BY created_on_ts DESC) AS rn
				FROM v_coin_store_events_grouped AS t1
			)
			WHERE
				rn = 1
    )
  SELECT
		a.grp_id, a.grp_span, a.grp_frame,
	  a.obj_id, l.obj_state, l.person, a.coins
	FROM aggr AS a
	JOIN latest AS l
		ON l.grp_id = a.grp_id AND l.obj_id = a.obj_id
	ORDER BY a.grp_id;
