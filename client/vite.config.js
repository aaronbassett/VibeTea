import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import tailwindcss from '@tailwindcss/vite';
import { compression } from 'vite-plugin-compression2';
export default defineConfig({
    plugins: [
        react(),
        tailwindcss(),
        compression({
            algorithms: ['brotliCompress'],
            exclude: [/\.(br)$/, /\.(gz)$/],
        }),
    ],
    build: {
        target: 'es2020',
        rollupOptions: {
            output: {
                manualChunks: {
                    'react-vendor': ['react', 'react-dom'],
                    state: ['zustand'],
                    virtual: ['@tanstack/react-virtual'],
                },
            },
        },
    },
    server: {
        port: 5173,
        proxy: {
            '/ws': {
                target: 'ws://localhost:8080',
                ws: true,
            },
        },
    },
});
