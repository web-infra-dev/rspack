const http = require("http");
/**
 * @returns {import("http").Server} server instance
 */
function createServer() {
  const server = http.createServer((req, res) => {
    const url = new URL(req.url, `http://${req.headers.host}`);

    if (url.pathname === "/redirect-to-disallowed") {
      res.writeHead(302, { "Location": "https://evil.com/malicious.js" });
      res.end();
      return;
    }

    if (url.pathname === "/redirect-to-non-http") {
      res.writeHead(302, { "Location": "ftp://example.com/file.js" });
      res.end();
      return;
    }

    if (url.pathname === "/redirect-chain") {
      res.writeHead(302, { "Location": "/redirect-chain-1" });
      res.end();
      return;
    }

    if (url.pathname === "/redirect-chain-1") {
      res.writeHead(302, { "Location": "/redirect-chain-2" });
      res.end();
      return;
    }

    if (url.pathname === "/redirect-chain-2") {
      res.writeHead(302, { "Location": "/redirect-chain-3" });
      res.end();
      return;
    }

    if (url.pathname === "/redirect-chain-3") {
      res.writeHead(302, { "Location": "/redirect-chain-4" });
      res.end();
      return;
    }

    if (url.pathname === "/redirect-chain-4") {
      res.writeHead(302, { "Location": "/redirect-chain-5" });
      res.end();
      return;
    }

    if (url.pathname === "/redirect-chain-5") {
      res.writeHead(302, { "Location": "/redirect-chain-6" });
      res.end();
      return;
    }

    res.writeHead(404);
    res.end("Not found");
  });
  return server;
}

class ServerPlugin {
  /**
   * @param {number} port
   */
  constructor(port) {
    this.port = port;
    this.refs = 0;
    this.server = undefined;
  }

  /**
   * @param {import("@rspack/core").Compiler} compiler
   */
  apply(compiler) {
    compiler.hooks.beforeRun.tapPromise(
      "ServerPlugin",
      () => {
        this.refs++;
        if (!this.server) {
          this.server = createServer();
          return new Promise((resolve, reject) => {
            this.server.listen(this.port, err => {
              if (err) {
                reject(err);
              } else {
                resolve();
              }
            });
          });
        }
      }
    );

    compiler.hooks.done.tap("ServerPlugin", (stats, callback) => {
      const s = this.server;
      if (s && --this.refs === 0) {
        this.server = undefined;
        s.close(callback);
      } else {
        callback();
      }
    });
  }
}

module.exports = ServerPlugin;
