// Custom HTTP client to mock server responses
// This is used by the buildHttp plugin to fetch resources

// Store all requests made for testing
const requests = [];

// Mock file contents - this simulates the server's response
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

// Custom HTTP client function - mimics the interface expected by rspack
const http_client = async (url, headers) => {
  // Extract pathname from URL for mocking
  const parsedUrl = new URL(url);
  const pathname = parsedUrl.pathname;

  // Log this request for testing
  requests.push({
    url,
    headers,
    timestamp: Date.now()
  });

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
