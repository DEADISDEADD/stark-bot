import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import path from 'path';

// Docker: backend:8082 (API), backend:8081 (WS), frontend on 8080
// Local:  localhost:8080 (API), localhost:8081 (WS), frontend on 5173
const isDocker = process.env.NODE_ENV === 'development' && process.env.DOCKER === '1';
const apiTarget = isDocker ? 'http://backend:8082' : 'http://localhost:8080';
const wsTarget = isDocker ? 'ws://backend:8081' : 'ws://localhost:8081';
const serverPort = isDocker ? 8080 : 5173;

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src')
    }
  },
  server: {
    port: serverPort,
    host: isDocker ? '0.0.0.0' : 'localhost',
    proxy: {
      '/api': apiTarget,
      '/ws': {
        target: wsTarget,
        ws: true,
        changeOrigin: true,
        rewritePath: (path) => path.replace(/^\/ws/, '')
      }
    }
  }
});
