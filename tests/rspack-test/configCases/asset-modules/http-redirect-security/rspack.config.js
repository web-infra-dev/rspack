/** @type {import("@rspack/core").Configuration} */
module.exports = {
  target: 'web',
  entry: './index.js',
  optimization: {
    moduleIds: 'named'
  },
  experiments: {
    buildHttp: {
      frozen: false,
      allowedUris: [
        "http://localhost:9991/"
      ],
      // Mock HTTP client to avoid real server and network issues
      httpClient: async (url, headers) => {
        // Mock redirect chain responses
        if (url === "http://localhost:9991/redirect-chain") {
          return {
            status: 302,
            headers: { location: "/redirect-chain-1" },
            body: Buffer.from("")
          };
        }

        if (url === "http://localhost:9991/redirect-chain-1") {
          return {
            status: 302,
            headers: { location: "/redirect-chain-2" },
            body: Buffer.from("")
          };
        }

        if (url === "http://localhost:9991/redirect-chain-2") {
          return {
            status: 302,
            headers: { location: "/redirect-chain-3" },
            body: Buffer.from("")
          };
        }

        if (url === "http://localhost:9991/redirect-chain-3") {
          return {
            status: 302,
            headers: { location: "/redirect-chain-4" },
            body: Buffer.from("")
          };
        }

        if (url === "http://localhost:9991/redirect-chain-4") {
          return {
            status: 302,
            headers: { location: "/redirect-chain-5" },
            body: Buffer.from("")
          };
        }

        if (url === "http://localhost:9991/redirect-chain-5") {
          return {
            status: 302,
            headers: { location: "/redirect-chain-6" },
            body: Buffer.from("")
          };
        }

        // Mock redirect to disallowed URL
        if (url === "http://localhost:9991/redirect-to-disallowed") {
          return {
            status: 302,
            headers: { location: "https://evil.com/malicious.js" },
            body: Buffer.from("")
          };
        }

        // Mock redirect to non-HTTP protocol
        if (url === "http://localhost:9991/redirect-to-non-http") {
          return {
            status: 302,
            headers: { location: "ftp://example.com/file.js" },
            body: Buffer.from("")
          };
        }

        // Default 404 response
        return {
          status: 404,
          headers: {},
          body: Buffer.from("Not found")
        };
      }
    }
  }
};
