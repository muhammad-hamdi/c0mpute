import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  reactStrictMode: true,
  // c0mpute.com landing — served at the apex. Per-plugin dashboards
  // (transcode, coinpay, infernet) will mount as separate Next apps under
  // /transcode, /coinpay, /infernet once we build them; for v1 the CLI is
  // the entire UX.
  trailingSlash: false,

  // 301 www.c0mpute.com → c0mpute.com (any path). Keeps the canonical
  // hostname clean; lets us configure TLS / DNS once at the apex without
  // an extra cert dance for the www subdomain.
  async redirects() {
    return [
      {
        source: "/:path*",
        has: [{ type: "host", value: "www.c0mpute.com" }],
        destination: "https://c0mpute.com/:path*",
        permanent: true,
      },
    ];
  },
};

export default nextConfig;
