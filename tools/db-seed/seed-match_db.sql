-- match_db seed
INSERT INTO matches (id, room_id, mode, status, winner_team, scheduled_at, started_at, ended_at) VALUES
  ('99991111-1111-1111-1111-111111111111', 'room-001', '5v5', 'finished', 'blue', NOW() - INTERVAL '2 hours', NOW() - INTERVAL '2 hours', NOW() - INTERVAL '1 hour 30 minutes'),
  ('99992222-2222-2222-2222-222222222222', 'room-002', '2v2', 'waiting',  NULL,   NOW() + INTERVAL '30 minutes', NULL, NULL);

INSERT INTO match_participants (id, match_id, player_id, team, score, kills, deaths, assists, is_mvp) VALUES
  ('88881111-1111-1111-1111-111111111111', '99991111-1111-1111-1111-111111111111', '11111111-1111-1111-1111-111111111111', 'blue', 12, 8, 3, 4, true),
  ('88882222-2222-2222-2222-222222222222', '99991111-1111-1111-1111-111111111111', '22222222-2222-2222-2222-222222222222', 'blue',  7, 5, 5, 6, false),
  ('88883333-3333-3333-3333-333333333333', '99991111-1111-1111-1111-111111111111', '33333333-3333-3333-3333-333333333333', 'red',   5, 4, 7, 3, false),
  ('88884444-4444-4444-4444-444444444444', '99992222-2222-2222-2222-222222222222', '44444444-4444-4444-4444-444444444444', 'blue',  0, 0, 0, 0, false);
