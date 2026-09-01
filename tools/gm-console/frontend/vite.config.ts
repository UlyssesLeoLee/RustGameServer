import { defineConfig, loadEnv } from 'vite';
import react from '@vitejs/plugin-react';
import { fileURLToPath } from 'url';
import { dirname, resolve } from 'path';

// RGS gm-backend 默认端口 8443 (HTTPS); dev 模式 RGS_ALLOW_INSECURE_GRPC=1 可走 8080 (待 main.rs 调整)
// 实际 proxy target 由 .env 或 shell env GM_BACKEND_URL 控制
export default defineConfig(({ mode }) => {
  const root = resolve(dirname(fileURLToPath(import.meta.url)), '..', '..');
  const env = loadEnv(mode, root, '');
  const target = env.GM_BACKEND_URL || 'http://localhost:8443';

  return {
    plugins: [react()],
    server: {
      port: 5173,
      proxy: {
        '/gm': { target, changeOrigin: true, secure: false },
        '/metrics': { target, changeOrigin: true, secure: false }
      }
    }
  };
});
