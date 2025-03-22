/**
 * Simple ServerPlugin for webpack config that mocks HTTP server behavior
 * without requiring an actual HTTP server.
 */

class ServerPlugin {
  constructor(port) {
    // Port is kept for API compatibility but not actually used
    this.port = port || 8999;
  }

  apply(compiler) {
    compiler.hooks.beforeRun.tap("ServerPlugin", () => {
      console.log("Test server initialized (default-uris test)");

      // If there's a custom HTTP client with request tracking, use it
      if (compiler.options.experiments?.buildHttp?.http_client?.clearRequests) {
        compiler.options.experiments.buildHttp.http_client.clearRequests();
      }
    });
  }
}

module.exports = ServerPlugin;
