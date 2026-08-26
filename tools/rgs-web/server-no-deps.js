const http = require('http');
const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

const PORT = process.env.RGS_WEB_PORT || 8787;

const routes = {
  '/api/health': () => ({
    status: 'ok',
    k3s: 'https://127.0.0.1:6443',
    time: new Date().toISOString(),
  }),
  '/api/impl-plan': () => {
    const dir = path.join(__dirname, '..', '..', 'docs', '12-工作流');
    const files = fs.readdirSync(dir).filter(f => f.startsWith('RGS-IMPL-PLAN-') && f.endsWith('.md'));
    return {
      plans: files.map(f => {
        const c = fs.readFileSync(path.join(dir, f), 'utf8');
        const status = (c.match(/\| 状态\s*\|\s*([^|]+)/) || [])[1]?.trim() || 'unknown';
        const owner = (c.match(/\| owner\s*\|\s*([^|]+)/) || [])[1]?.trim() || 'unknown';
        return { file: f, status, owner, size: c.length };
      })
    };
  },
  '/api/worktrees': () => {
    try {
      const out = execSync('git worktree list --porcelain', { cwd: 'D:/RustGameServer', encoding: 'utf8' });
      const wts = out.split('\n\n').filter(s => s.trim()).map(block => {
        const lines = block.split('\n');
        return {
          path: lines.find(l => l.startsWith('worktree '))?.split(' ').slice(1).join(' '),
          head: lines.find(l => l.startsWith('HEAD '))?.split(' ')[1]?.substring(0, 7),
          branch: lines.find(l => l.startsWith('branch '))?.split(' ').slice(1).join(' ')?.replace('refs/heads/', ''),
        };
      });
      return { worktrees: wts };
    } catch (e) { return { error: e.message }; }
  },
  '/api/docs-health': () => ({
    fail: 1, warn: 1,
    fail_reason: 'RGS-DEC-NOGO-001 缺决策编号字段',
    warn_reason: '5 ADR 待具名审批',
    last_check: '2026-08-26 04:30 JST 基线',
    p0p1p2_commits: 11,
  }),
};

const server = http.createServer((req, res) => {
  if (req.url === '/' || req.url === '/index.html') {
    res.writeHead(200, { 'Content-Type': 'text/html; charset=utf-8' });
    return fs.createReadStream(path.join(__dirname, 'public', 'index.html')).pipe(res);
  }
  const handler = routes[req.url];
  if (handler) {
    res.writeHead(200, { 'Content-Type': 'application/json; charset=utf-8' });
    return res.end(JSON.stringify(handler()));
  }
  res.writeHead(404);
  res.end('not found');
});

server.listen(PORT, '127.0.0.1', () => {
  console.log('RGS Admin Web (no-express) 启动: http://127.0.0.1:' + PORT);
});
