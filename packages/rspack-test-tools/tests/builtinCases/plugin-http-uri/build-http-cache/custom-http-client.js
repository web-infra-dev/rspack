// Custom HTTP client for cache testing
const ServerPlugin = require('./server.js');

// Store all requests made for testing
const requests = [];

// Mock file contents with cache control settings
const mockFiles = {
  "/module.js": {
    status: 200,
    headers: {
      "content-type": "application/javascript",
      "cache-control": "public, immutable, max-age=600"
    },
    content: `module.exports = "Module loaded ${ServerPlugin.incrementRequestCount()} times";`
  },
  "/module-etag.js": {
    status: 200,
    headers: {
      "content-type": "application/javascript",
      "etag": "\"test-etag-value\"",
      "cache-control": "public, max-age=600"
    },
    content: `module.exports = "Module with ETag loaded ${ServerPlugin.incrementRequestCount()} times";`
  },
  "/module-no-cache.js": {
    status: 200,
    headers: {
      "content-type": "application/javascript",
      "cache-control": "no-cache, no-store"
    },
    content: `module.exports = "No-cache module loaded ${ServerPlugin.incrementRequestCount()} times";`
  },
  "/module-max-age.js": {
    status: 200,
    headers: {
      "content-type": "application/javascript",
      "cache-control": "max-age=60"
    },
    content: `module.exports = "Max-age module loaded ${ServerPlugin.incrementRequestCount()} times";`
  }
};

// Custom HTTP client function for cache testing
const http_client = async (url, headers) => {
  // Extract pathname from URL for mocking
  const parsedUrl = new URL(url);
  const pathname = parsedUrl.pathname;

  // Log this request for testing
  const request = {
    url,
    headers,
    timestamp: Date.now()
  };
  requests.push(request);

  // Check if we have a mock for this file
  if (mockFiles[pathname]) {
    const mockFile = mockFiles[pathname];

    // Handle ETag caching
    if (mockFile.headers.etag && headers["if-none-match"] === mockFile.headers.etag) {
      // Return 304 Not Modified
      return {
        status: 304,
        headers: mockFile.headers,
        body: Buffer.from("")
      };
    }

    // Simulate network delay
    await new Promise(resolve => setTimeout(resolve, 10));

    // Return the mock response
    return {
      status: mockFile.status,
      headers: mockFile.headers,
      body: Buffer.from(mockFile.content)
    };
  }

  // If there's no mock, return a 404
  return {
    status: 404,
    headers: { "content-type": "text/plain" },
    body: Buffer.from(`Not found: ${pathname}`)
  };
};

// Function to get all tracked requests
const getRequests = () => {
  return [...requests];
};

// Function to clear the request tracking
const clearRequests = () => {
  requests.length = 0;
};

// Export the client and utility functions
module.exports = http_client;
module.exports.getRequests = getRequests;
module.exports.clearRequests = clearRequests;
