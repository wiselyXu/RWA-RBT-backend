import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  server: {
    proxy: {
      // Proxy /rwa requests to your backend server
      '/rwa': {
        target: 'http://127.0.0.1:8888', // Your backend address
        changeOrigin: true,
        // rewrite: (path) => path.replace(/^\/rwa/, '') // Uncomment if backend doesn't expect /rwa prefix
      }
    }
  }
})
