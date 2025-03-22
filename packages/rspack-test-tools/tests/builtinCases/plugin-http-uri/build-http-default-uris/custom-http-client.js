// Custom HTTP client for build-http-default-uris test

// Store all requests made for testing
const requests = [];

// Mock file contents
const mockFiles = {
  "/module.js": {
    status: 200,
    headers: {
      "content-type": "application/javascript"
    },
    content: "module.exports = 'DEFAULT_URIS_MODULE';"
  },
  "/allowed-module.js": {
    status: 200,
    headers: {
      "content-type": "application/javascript"
    },
    content: "module.exports = 'ALLOWED_MODULE';"
  },
  "/disallowed-module.js": {
    status: 200,
    headers: {
      "content-type": "application/javascript"
    },
    content: "module.exports = 'DISALLOWED_MODULE';"
  }
};

// Custom HTTP client function
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

  console.log(`🔍 Default-uris test received request for: ${url}`);

  // Check if we have a mock for this file
  if (mockFiles[pathname]) {
    const mockFile = mockFiles[pathname];

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
