// RGS Admin Web — 代理 k3s API + 静态 HTML Dashboard
// per RGS-WEB-PLAN-2026-08-26 v0.1
const express = require('express');
const path = require('path');
const https = require('https');
const fs = require('fs');

const PORT = process.env.RGS_WEB_PORT || 8788;
const K3S_API = process.env.K3S_API || 'https://127.0.0.1:6443';
const K3S_TOKEN = process.env.K3S_TOKEN || '';  // 需 sudo chmod 后用
const K3S_CA = process.env.K3S_CA_PATH || '';   // /etc/rancher/k3s/server/tls/server-ca.crt

const app = express();
app.use(express.json());
app.use(express.static(path.join(__dirname, 'public')));

// Health
app.get('/api/health', (req, res) => {
  res.json({ status: 'ok', k3s: K3S_API, time: new Date().toISOString() });
});

// 代理 k3s API(透传 path + auth)
app.use('/api/k8s', async (req, res) => {
  const path = req.url;
  const target = K3S_API + path;
  try {
    const ca = K3S_CA ? fs.readFileSync(K3S_CA) : undefined;
    const opts = {
      method: req.method,
      headers: { 'Authorization': `Bearer ${K3S_TOKEN}` },
      ca,
      rejectUnauthorized: K3S_CA ? true : false,
    };
    const proxyReq = https.request(target, opts, (proxyRes) => {
      res.status(proxyRes.statusCode);
      for (const [k, v] of Object.entries(proxyRes.headers)) res.setHeader(k, v);
      proxyRes.pipe(res);
    });
    proxyReq.on('error', (e) => res.status(502).json({ error: e.message }));
    req.pipe(proxyReq);
  } catch (e) {
    res.status(500).json({ error: e.message });
  }
});

// 5 域 IMPL-PLAN 状态(从本地 RGS-IMPL-PLAN-*-V1 文件读取进度)
app.get('/api/impl-plan', (req, res) => {
  const dir = path.join(__dirname, '..', '..', 'docs', '12-工作流');
  try {
    const files = fs.readdirSync(dir).filter(f => f.startsWith('RGS-IMPL-PLAN-') && f.endsWith('.md'));
    const plans = files.map(f => {
      const content = fs.readFileSync(path.join(dir, f), 'utf8');
      const status = (content.match(/\| 状态\s*\|\s*([^|]+)/) || [])[1]?.trim() || 'unknown';
      const owner = (content.match(/\| owner\s*\|\s*([^|]+)/) || [])[1]?.trim() || 'unknown';
      return { file: f, status, owner, size: content.length };
    });
    res.json({ plans });
  } catch (e) {
    res.status(500).json({ error: e.message });
  }
});

// 文档健康(per check-docs-consistency.sh 基线)
app.get('/api/docs-health', (req, res) => {
  res.json({
    fail: 1,
    warn: 1,
    fail_reason: 'RGS-DEC-NOGO-001 缺决策编号字段',
    warn_reason: '5 ADR 待具名审批',
    last_check: '2026-08-26 04:30 JST 基线',
    p0p1p2_commits: 11,
  });
});

// 11 个 worktree 状态(从 git 读)
app.get('/api/worktrees', (req, res) => {
  const { execSync } = require('child_process');
  try {
    const out = execSync('git worktree list --porcelain', { cwd: 'D:/RustGameServer', encoding: 'utf8' });
    const wts = out.split('\n\n').filter(s => s.trim()).map(block => {
      const lines = block.split('\n');
      const path = lines.find(l => l.startsWith('worktree '))?.split(' ')[1];
      const head = lines.find(l => l.startsWith('HEAD '))?.split(' ')[1]?.substring(0, 7);
      const branch = lines.find(l => l.startsWith('branch '))?.split(' ')[1]?.replace('refs/heads/', '');
      return { path, head, branch };
    });
    res.json({ worktrees: wts });
  } catch (e) {
    res.status(500).json({ error: e.message });
  }
});

app.listen(PORT, '127.0.0.1', () => {
  console.log(`RGS Admin Web 启动: http://127.0.0.1:${PORT}`);
  console.log(`  k3s API: ${K3S_API}`);
  console.log(`  Token: ${K3S_TOKEN ? K3S_TOKEN.substring(0, 10) + '...' : '(unset - kubectl 503)'}`);
  console.log(`  CA: ${K3S_CA || '(using skip-verify)'}`);
});
