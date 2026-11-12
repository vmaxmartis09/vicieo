import createNextIntlPlugin from "next-intl/plugin";

const withNextIntl = createNextIntlPlugin("./i18n/request.ts");

/** @type {import('next').NextConfig} */
const nextConfig = {
  ...(process.env.NEXT_DIST_DIR ? { distDir: process.env.NEXT_DIST_DIR } : {}),
  transpilePackages: ["morphicons"],
  reactStrictMode: true,
  devIndicators: { position: "bottom-right" },
  images: {
    unoptimized: true,
  },
  async rewrites() {
    return [
      { source: "/projects/:path*", destination: "http://127.0.0.1:9000/projects/:path*" },
      { source: "/data/:path*", destination: "http://127.0.0.1:9000/data/:path*" },
      { source: "/voices/:path*", destination: "http://127.0.0.1:9000/voices/:path*" },
      { source: "/health", destination: "http://127.0.0.1:9000/health" },
      { source: "/node-types", destination: "http://127.0.0.1:9000/node-types" },
      { source: "/custom-node-types", destination: "http://127.0.0.1:9000/custom-node-types" },
      { source: "/models", destination: "http://127.0.0.1:9000/models" },
      { source: "/providers", destination: "http://127.0.0.1:9000/providers" },
      { source: "/graph/:path*", destination: "http://127.0.0.1:9000/graph/:path*" },
      { source: "/video/:path*", destination: "http://127.0.0.1:9000/video/:path*" },
      { source: "/voice/:path*", destination: "http://127.0.0.1:9000/voice/:path*" },
    ];
  },
};

export default withNextIntl(nextConfig);
