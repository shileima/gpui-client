import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import federation from '@originjs/vite-plugin-federation';

const BUILD_TS = Date.now();
const GPUI_BRIDGE = process.env.GPUI_BRIDGE_URL || 'http://127.0.0.1:38472';

export default defineConfig({
  plugins: [
    react(),
    federation({
      name: 'gpui_chat_host',
      remotes: {
        aichat: {
          external: `Promise.resolve((window.__AICHAT_REMOTE_URL__ || 'https://aichat.sankuai.com/remoteEntry.js') + '?v=${BUILD_TS}')`,
          externalType: 'promise',
          format: 'var',
          from: 'webpack',
        },
      },
      shared: {
        react: { singleton: true, requiredVersion: '^18.3.1' },
        'react-dom': { singleton: true, requiredVersion: '^18.3.1' },
        antd: { singleton: true },
      },
    }),
  ],
  server: {
    port: 5173,
    strictPort: true,
    cors: true,
    proxy: {
      '/__gpui_bridge': GPUI_BRIDGE,
    },
  },
  build: {
    target: 'esnext',
    minify: false,
    cssCodeSplit: false,
    outDir: 'dist',
  },
});
