import type { NextConfig } from 'next';

const nextConfig: NextConfig = {
  turbopack: {
    rules: {
      '*.css': {
        loaders: ['@tailwindcss/turbopack']
      }
    }
  },

  output: 'export',
  trailingSlash: true,
  images: { unoptimized: true }
};

export default nextConfig;
