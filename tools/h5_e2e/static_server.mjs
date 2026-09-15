// tools/h5_e2e/static_server.mjs
//
// Tiny static-file server for the puppeteer (Playwright) E2E page.
// Serves tools/h5_e2e/* at http://127.0.0.1:18001/.
//
// Run:
//   node tools/h5_e2e/static_server.mjs
// then load http://127.0.0.1:18001/smart_socket_test.html in a browser.

import http from 'node:http';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const PORT = Number(process.env.STATIC_PORT || 18001);
const HOST = process.env.STATIC_HOST || '0.0.0.0';

const MIME = {
  '.html': 'text/html; charset=utf-8',
  '.js': 'application/javascript; charset=utf-8',
  '.mjs': 'application/javascript; charset=utf-8',
  '.json': 'application/json; charset=utf-8',
  '.css': 'text/css; charset=utf-8',
  '.txt': 'text/plain; charset=utf-8',
  '.png': 'image/png',
  '.svg': 'image/svg+xml',
};

const server = http.createServer((req, res) => {
  let url = req.url.split('?')[0];
  if (url === '/' || url === '') url = '/smart_socket_test.html';
  const filePath = path.join(__dirname, url);
  // prevent directory traversal
  if (!filePath.startsWith(__dirname)) {
    res.writeHead(403); res.end('forbidden'); return;
  }
  fs.readFile(filePath, (err, data) => {
    if (err) {
      res.writeHead(404, { 'Content-Type': 'text/plain' });
      res.end('not found: ' + url);
      return;
    }
    const ext = path.extname(filePath).toLowerCase();
    res.writeHead(200, { 'Content-Type': MIME[ext] || 'application/octet-stream' });
    res.end(data);
  });
});

server.listen(PORT, HOST, () => {
  console.log(`[static] serving ${__dirname} at http://${HOST}:${PORT}/`);
  console.log(`[static] open http://127.0.0.1:${PORT}/smart_socket_test.html`);
});

process.on('SIGINT', () => { server.close(); process.exit(0); });