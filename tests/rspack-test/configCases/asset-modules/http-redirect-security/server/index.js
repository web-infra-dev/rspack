const http = require("http");
/**
 * @returns {import("http").Server} server instance
 */
function createServer() {
  const server = http.createServer((req, res) => {
    const url = new URL(req.url, `http://${req.headers.host}`);
    const logOutcome = (status, headers) => {
      const location = headers && headers.Location ? ` Location=${headers.Location}` : "";
      console.log(`[server] -> ${status}${location}`);
    };
    console.log(`[server] <- ${req.method} ${url.pathname}`);

    if (url.pathname === "/redirect-to-disallowed") {
      const headers = { "Location": "https://evil.com/malicious.js" };
      res.writeHead(302, headers);
      logOutcome(302, headers);
      res.end();
      return;
    }

    if (url.pathname === "/redirect-to-non-http") {
      const headers = { "Location": "ftp://example.com/file.js" };
      res.writeHead(302, headers);
      logOutcome(302, headers);
      res.end();
      return;
    }

    if (url.pathname === "/redirect-chain") {
      const headers = { "Location": "/redirect-chain-1" };
      res.writeHead(302, headers);
      logOutcome(302, headers);
      res.end();
      return;
    }

    if (url.pathname === "/redirect-chain-1") {
      const headers = { "Location": "/redirect-chain-2" };
      res.writeHead(302, headers);
      logOutcome(302, headers);
      res.end();
      return;
    }

    if (url.pathname === "/redirect-chain-2") {
      const headers = { "Location": "/redirect-chain-3" };
      res.writeHead(302, headers);
      logOutcome(302, headers);
      res.end();
      return;
    }

    if (url.pathname === "/redirect-chain-3") {
      const headers = { "Location": "/redirect-chain-4" };
      res.writeHead(302, headers);
      logOutcome(302, headers);
      res.end();
      return;
    }

    if (url.pathname === "/redirect-chain-4") {
      const headers = { "Location": "/redirect-chain-5" };
      res.writeHead(302, headers);
      logOutcome(302, headers);
      res.end();
      return;
    }

    if (url.pathname === "/redirect-chain-5") {
      const headers = { "Location": "/redirect-chain-6" };
      res.writeHead(302, headers);
      logOutcome(302, headers);
      res.end();
      return;
    }

    res.writeHead(404);
    logOutcome(404);
    res.end("Not found");
  });
  server.unref();
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

    compiler.hooks.done.tapAsync("ServerPlugin", (stats, callback) => {

      console.log('callback', callback, stats)

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
