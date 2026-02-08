const httpClient = async (url) => {
  const parsedUrl = new URL(url);
  const pathname = parsedUrl.pathname;

  if (pathname === "/redirect-to-disallowed") {
    return {
      status: 302,
      headers: { "location": "https://evil.com/malicious.js" },
      body: Buffer.from("")
    };
  }

  if (pathname === "/redirect-to-non-http") {
    return {
      status: 302,
      headers: { "location": "ftp://example.com/file.js" },
      body: Buffer.from("")
    };
  }

  if (pathname === "/redirect-chain") {
    return {
      status: 302,
      headers: { "location": "http://localhost:9991/redirect-chain-1" },
      body: Buffer.from("")
    };
  }

  if (pathname === "/redirect-chain-1") {
    return {
      status: 302,
      headers: { "location": "http://localhost:9991/redirect-chain-2" },
      body: Buffer.from("")
    };
  }

  if (pathname === "/redirect-chain-2") {
    return {
      status: 302,
      headers: { "location": "http://localhost:9991/redirect-chain-3" },
      body: Buffer.from("")
    };
  }

  if (pathname === "/redirect-chain-3") {
    return {
      status: 302,
      headers: { "location": "http://localhost:9991/redirect-chain-4" },
      body: Buffer.from("")
    };
  }

  if (pathname === "/redirect-chain-4") {
    return {
      status: 302,
      headers: { "location": "http://localhost:9991/redirect-chain-5" },
      body: Buffer.from("")
    };
  }

  if (pathname === "/redirect-chain-5") {
    return {
      status: 302,
      headers: { "location": "http://localhost:9991/redirect-chain-6" },
      body: Buffer.from("")
    };
  }

  return {
    status: 404,
    headers: { "content-type": "text/plain" },
    body: Buffer.from("Not found")
  };
};

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
      httpClient
    }
  }
};
