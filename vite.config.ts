import react from "@vitejs/plugin-react-swc";
import { defineConfig } from "vite";

const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig({
  plugins: [react()],

  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
        protocol: "ws",
        host,
        port: 1421,
      }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
  build: {
    rolldownOptions: {
      output: {
        codeSplitting: {
          groups: [
            { name: 'antd', test: /antd/ },
            { name: "xterm", test: /@xterm\/.+/ },
            { name: "monaco", test: /@xterm-editor\/.+|monaco-editor/ },
          ]
        }
      }
    }
  }
});
