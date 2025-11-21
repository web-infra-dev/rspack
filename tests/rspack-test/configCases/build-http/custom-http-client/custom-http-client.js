const mockFiles = {
  "/allowed-module.js": {
    status: 200,
    headers: { "content-type": "application/javascript" },
    content: 'module.exports = "This module is from an allowed URI";'
  },
  "/regex-module.js": {
    status: 200,
    headers: { "content-type": "application/javascript" },
    content: 'module.exports = "This module is from a regex-matched URI";'
  },
  "/restricted-module.js": {
    status: 200,
    headers: { "content-type": "application/javascript" },
    content: 'module.exports = "This module is from a restricted URI";'
  }
};

const httpClient = async (url, headers) => {
  const parsedUrl = new URL(url);
  const pathname = parsedUrl.pathname;

  if (mockFiles[pathname]) {
    const mockFile = mockFiles[pathname];

    return {
      status: mockFile.status,
      headers: mockFile.headers,
      body: Buffer.from(mockFile.content)
    };
  }

  return {
    status: 404,
    headers: { "content-type": "text/plain" },
    body: Buffer.from(`Not found: ${pathname}`)
  };
};

module.exports = httpClient;
