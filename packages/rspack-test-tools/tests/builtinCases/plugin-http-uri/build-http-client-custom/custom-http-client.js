// Track requests for testing
let requests = [];

/**
 * Custom HTTP client implementation that logs all requests
 * and returns mock responses for testing
 */
module.exports = function customHttpClient(url, headers) {
  // Track this request
  requests.push({ url, headers });

  // Simple mock server implementation
  if (url.startsWith("http://localhost") || url.startsWith("https://localhost")) {
    let content;

    // Mock responses based on URL
    if (url.endsWith("/module.js") || url === "http://localhost/") {
      content = 'export default "Module from custom HTTP client";';
    } else if (url.includes("/dependency.js")) {
      content = 'export default "Dependency module";';
    } else {
      // 404 for unknown paths
      return Promise.resolve({
        status: 404,
        headers: {},
        body: Buffer.from(`Not found: ${url}`)
      });
    }

    // Return a successful response
    return Promise.resolve({
      status: 200,
      headers: {
        "content-type": "application/javascript",
        "x-custom-header": "custom-value",
        "cache-control": "max-age=3600"
      },
      body: Buffer.from(content)
    });
  }

  // For real URLs, return a promise that would use the JS-provided fetch API
  return Promise.resolve().then(() => {
    // This part would normally use the provided fetch implementation
    // but we're returning a mock for the test
    return {
      status: 200,
      headers: {
        "content-type": "application/javascript"
      },
      body: Buffer.from('export default "Fetched module";')
    };
  });
};

// Expose for testing
module.exports.getRequests = function() {
  return requests;
};

module.exports.clearRequests = function() {
  requests = [];
};
