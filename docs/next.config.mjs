import { createMDX } from 'fumadocs-mdx/next';

const withMDX = createMDX();

const isGithubActions = process.env.GITHUB_ACTIONS === 'true';

/** @type {import('next').NextConfig} */
const config = {
  output: 'export',
  basePath: isGithubActions ? '/LaadleLang' : undefined,
  assetPrefix: isGithubActions ? '/LaadleLang' : undefined,
  images: {
    unoptimized: true,
  },
  serverExternalPackages: ['@takumi-rs/image-response'],
  reactStrictMode: true,
};

export default withMDX(config);