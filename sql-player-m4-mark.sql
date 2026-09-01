-- mark m4 as already run
INSERT INTO _sqlx_migrations (version, description, installed_on, success, checksum, execution_time)
VALUES (4, 'player_characters_inventory', now(), true, decode('00', 'hex'), 0)
ON CONFLICT (version) DO NOTHING;
