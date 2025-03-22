/**
 * Simple ServerPlugin for webpack config that mocks HTTP caching behavior
 * without requiring an actual HTTP server.
 */

// Counter to track how many times resources have been requested
let requestCount = 0;

/**
 * Server Plugin that tracks request counts for cache testing
 */
class ServerPlugin {
  constructor(port) {
    // Port is kept for API compatibility but not actually used
    this.port = port || 8999;
  }

  apply(compiler) {
    // Reset request count at the start of each compilation
    compiler.hooks.beforeRun.tap("ServerPlugin", () => {
      requestCount = 0;

      // If there's a custom HTTP client with request tracking, use it
      if (compiler.options.experiments?.buildHttp?.http_client?.clearRequests) {
        compiler.options.experiments.buildHttp.http_client.clearRequests();
      }

      console.log("Cache test initialized with request count = 0");
    });
  }

  // Expose request count for testing
  static getRequestCount() {
    return requestCount;
  }

  // Increment request count - can be called from custom HTTP client
  static incrementRequestCount() {
    return ++requestCount;
  }
}

module.exports = ServerPlugin;
