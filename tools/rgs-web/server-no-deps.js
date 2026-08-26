// RGS Admin Web — 零依赖版(GM 后台 12 页面)
// per RGS-WEB-GM-PLAN-2026-08-26 v0.1
const http = require('http');
const https = require('https');
const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

const PORT = process.env.RGS_WEB_PORT || 8788;
const K3S_API = process.env.K3S_API || 'https://127.0.0.1:6443';
const K3S_TOKEN = process.env.K3S_TOKEN || '';
const K3S_CA = process.env.K3S_CA_PATH || '';

const handlers = {
  '/api/health': () => ({
    status: 'ok', k3s: K3S_API, time: new Date().toISOString(), pid: process.pid,
    rgs_web_version: '0.2.0-gm',
    pages: ['dashboard','servers','players','stream','config','hotupdate','operations','docs','worktrees','reports'],
  }),

  '/api/impl-plan': () => {
    // 跨 worktree 找所有 IMPL-PLAN
    const wtOut = execSync('git worktree list --porcelain', { cwd: 'D:/RustGameServer', encoding: 'utf8' });
    const wts = wtOut.split('\n\n').filter(s => s.trim()).map(block => {
      return block.split('\n').find(l => l.startsWith('worktree '))?.split(' ').slice(1).join(' ');
    });
    let all = [];
    for (const wt of wts) {
      const dir = path.join(wt, 'docs/12-工作流');
      if (!fs.existsSync(dir)) continue;
      const files = fs.readdirSync(dir).filter(f => f.startsWith('RGS-IMPL-PLAN-') && f.endsWith('.md'));
      for (const f of files) {
        const c = fs.readFileSync(path.join(dir, f), 'utf8');
        const status = (c.match(/\| 状态\s*\|\s*([^|]+)/) || [])[1]?.trim() || 'unknown';
        const owner = (c.match(/\| owner\s*\|\s*([^|]+)/) || [])[1]?.trim() || 'unknown';
        const wname = wt.replace('D:/RustGameServer-worktrees/', '').replace('D:/RustGameServer', 'main');
        all.push({ file: f, status, owner, size: c.length, worktree: wname });
      }
    }
    const dedup = [];
    const seen = new Set();
    for (const f of all) { if (!seen.has(f.file)) { seen.add(f.file); dedup.push(f); } }
    return { plans: dedup };
  },

  '/api/worktrees': () => {
    try {
      const out = execSync('git worktree list --porcelain', { cwd: 'D:/RustGameServer', encoding: 'utf8' });
      const wts = out.split('\n\n').filter(s => s.trim()).map(block => {
        const lines = block.split('\n');
        const path = lines.find(l => l.startsWith('worktree '))?.split(' ').slice(1).join(' ');
        const head = lines.find(l => l.startsWith('HEAD '))?.split(' ')[1]?.substring(0, 7);
        const branch = lines.find(l => l.startsWith('branch '))?.split(' ').slice(1).join(' ')?.replace('refs/heads/', '');
        // 判断 locked
        const isLocked = block.includes('locked');
        return { path, head, branch, locked: isLocked };
      });
      return { worktrees: wts, total: wts.length };
    } catch (e) { return { error: e.message }; }
  },

  '/api/docs-health': () => ({
    fail: 1, warn: 1,
    fail_reason: 'RGS-DEC-NOGO-001 缺决策编号字段',
    warn_reason: '5 ADR 待具名审批',
    last_check: '2026-08-26 04:30 JST 基线',
    p0p1p2_commits: 11,
  }),

  '/api/git-log': () => {
    try {
      const out = execSync('git log --pretty=format:"%h|%ad|%s" --date=short -n 30', { cwd: 'D:/RustGameServer', encoding: 'utf8' });
      const commits = out.split('\n').filter(l => l.trim()).map(l => {
        const [hash, date, ...msg] = l.split('|');
        return { hash, date, message: msg.join('|') };
      });
      return { commits, total: commits.length };
    } catch (e) { return { error: e.message }; }
  },

  '/api/saga-trace': () => {
    // Saga 域追踪(per RGS-IMPL-100)
    try {
      const out = execSync('git log --all --pretty=format:"%h|%ad|%s" --date=iso --grep="saga" -n 20', { cwd: 'D:/RustGameServer', encoding: 'utf8' });
      const commits = out.split('\n').filter(l => l.trim()).map(l => {
        const [hash, date, ...msg] = l.split('|');
        return { hash, date, message: msg.join('|') };
      });
      return { traces: commits };
    } catch (e) { return { error: e.message }; }
  },
};

// k3s API 代理(/api/k8s/* → 6443)
function proxyK3s(req, res) {
  const url = req.url.replace(/^\/api\/k8s/, '');
  const target = `${K3S_API}${url}`;
  const headers = { ...req.headers };
  if (K3S_TOKEN) headers['Authorization'] = `Bearer ${K3S_TOKEN}`;
  delete headers['host'];
  const opts = {
    method: req.method,
    headers,
    rejectUnauthorized: !!K3S_CA,
    ca: K3S_CA ? fs.readFileSync(K3S_CA) : undefined,
  };
  const proxyReq = https.request(target, opts, (proxyRes) => {
    res.writeHead(proxyRes.statusCode, proxyRes.headers);
    proxyRes.pipe(res);
  });
  proxyReq.on('error', (e) => {
    res.writeHead(502, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({ error: e.message, k3s: K3S_API, hint: 'WSL sudo chmod 644 /etc/rancher/k3s/k3s.yaml 之后 K3S_TOKEN 即可用' }));
  });
  req.pipe(proxyReq);
}

const server = http.createServer((req, res) => {
  if (req.url === '/' || req.url === '/index.html') {
    res.writeHead(200, { 'Content-Type': 'text/html; charset=utf-8' });
    return fs.createReadStream(path.join(__dirname, 'public', 'index.html')).pipe(res);
  }
  // 静态资源(public/*)
  if (req.url.startsWith('/public/') || req.url.match(/\.(css|js|svg|png|ico)$/)) {
    const fp = path.join(__dirname, req.url);
    if (fs.existsSync(fp)) {
      const ext = path.extname(fp).substring(1);
      const mime = { css: 'text/css', js: 'application/javascript', svg: 'image/svg+xml', png: 'image/png', ico: 'image/x-icon' }[ext] || 'application/octet-stream';
      res.writeHead(200, { 'Content-Type': mime });
      return fs.createReadStream(fp).pipe(res);
    }
  }
  // API 路由
  const handler = handlers[req.url];
  if (handler) {
    res.writeHead(200, { 'Content-Type': 'application/json; charset=utf-8' });
    return res.end(JSON.stringify(handler()));
  }
  // k3s 代理
  if (req.url.startsWith('/api/k8s')) return proxyK3s(req, res);
  // 404
  res.writeHead(404, { 'Content-Type': 'application/json' });
  res.end(JSON.stringify({ error: 'not found', path: req.url }));
});

server.listen(PORT, '127.0.0.1', () => {
  console.log(`RGS Admin Web v0.2-gm 启动: http://127.0.0.1:${PORT}`);
  console.log(`  k3s API: ${K3S_API}`);
  console.log(`  K3S_TOKEN: ${K3S_TOKEN ? '***set***' : '(unset — 代理 401)'}`);
  console.log(`  PID: ${process.pid}`);
});
