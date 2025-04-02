// This file simulates the content that would be returned from https://example.com/absolute-path-test.js
// It contains absolute paths that should be treated as local paths, not HTTP URLs

const mockFiles = {
  "/absolute-path-test.js": {
    status: 200,
    headers: { "content-type": "application/javascript" },
    content: `
      // Import the real module using the actual absolute path
      const realModule = require("/Users/bytedance/RustroverProjects/rspack/packages/rspack-test-tools/tests/configCases/build-http/absolute-path-in-http/real-module.js");

      module.exports = {
        message: realModule.message,
        getMessage: realModule.getMessage
      };
    `
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
