// Simple ServerPlugin for webpack config
class ServerPlugin {
  constructor() {}

  apply(compiler) {
    // This plugin doesn't do anything special in the tests
    // It's here to maintain consistency with the pattern used in other tests
    compiler.hooks.beforeRun.tap("ServerPlugin", () => {
      // Initialize any test state here if needed
      if (typeof require("./custom-http-client").clearRequests === "function") {
        require("./custom-http-client").clearRequests();
      }
    });
  }
}

// Export plugin for tests
module.exports = {
  ServerPlugin
}; 