-- cluster_ops_db seed
INSERT INTO cluster_nodes (id, hostname, ip, role, status, version) VALUES
  ('33331111-1111-1111-1111-111111111111', 'k3s-server-01', '10.42.0.1', 'primary',   'healthy', 'v1.0.0'),
  ('33332222-2222-2222-2222-222222222222', 'k3s-agent-01',  '10.42.0.7', 'candidate', 'healthy', 'v1.0.0');

INSERT INTO feature_flags (key, scope, scope_value, enabled, version, updated_by) VALUES
  ('metrics.scrape.enabled',    'global', '*',      true,  1, '33331111-1111-1111-1111-111111111111'),
  ('rgs.debug.verbose_logging', 'domain', 'player', false, 1, '33331111-1111-1111-1111-111111111111');
