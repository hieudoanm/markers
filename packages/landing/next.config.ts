import type { NextConfig } from 'next';

const NODE_ENV = process.env.NODE_ENV ?? 'development';

const nextConfig: NextConfig = {
  turbopack: {
    rules: {
      '*.css': {
        loaders: ['@tailwindcss/turbopack'],
      },
    },
  },

  output: 'export',
  trailingSlash: true,
  // GitHub Pages project site lives under /markers/
  basePath: NODE_ENV === 'production' ? '/markers' : '',
  assetPrefix: NODE_ENV === 'production' ? '/markers' : '',
  images: { unoptimized: true },
};

export default nextConfig;
