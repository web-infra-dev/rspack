// This file simulates the content that would be returned from https://example.com/absolute-path-test.js
// It contains absolute paths that should be treated as local paths, not HTTP URLs

const path = require('path');
const realModulePath = path.resolve(__dirname, 'real-module.js');

const mockFiles = {
  "/absolute-path-test.js": {
    status: 200,
    headers: { "content-type": "application/javascript" },
    content: `
      // Import the real module using the actual absolute path
      // Using dynamic import for ES module compatibility
      const realModule = require(${JSON.stringify(realModulePath)});

      // Export as ES module for React compatibility
      export const message = realModule.message;
      export const getMessage = realModule.getMessage;

      // For CommonJS compatibility
      module.exports = {
        message: realModule.message,
        getMessage: realModule.getMessage
      };
    `
  },
  "/react-component.js": {
    status: 200,
    headers: { "content-type": "application/javascript" },
    content: `
      // A remote React component to demonstrate full compatibility
      import React from 'https://esm.sh/react';

      export function RemoteComponent({ label }) {
        return React.createElement('div', { className: 'remote-component' },
          React.createElement('h3', null, 'Remote Component'),
          React.createElement('p', null, label || 'Default Label')
        );
      }

      export default RemoteComponent;
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
