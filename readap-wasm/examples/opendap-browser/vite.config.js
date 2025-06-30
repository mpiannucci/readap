import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// https://vite.dev/config/
export default defineConfig({
  plugins: [
    react(),
    // Custom plugin to serve WASM files with correct MIME type
    {
      name: 'wasm-mime-type',
      configureServer(server) {
        server.middlewares.use((req, res, next) => {
          if (req.url?.endsWith('.wasm')) {
            res.setHeader('Content-Type', 'application/wasm')
          }
          next()
        })
      }
    }
  ],
  optimizeDeps: {
    exclude: ['@mattnucc/readap']
  },
  assetsInclude: ['**/*.wasm']
})
