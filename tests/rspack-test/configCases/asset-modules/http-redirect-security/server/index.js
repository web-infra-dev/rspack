const http = require("http");
/**
 * @returns {import("http").Server} server instance
 */
function createServer() {
  const server = http.createServer((req, res) => {
    const url = new URL(req.url, `http://${req.headers.host}`);

    console.log("<", url.pathname);

    if (url.pathname === "/redirect-to-disallowed") {
      res.writeHead(302, { "Location": "https://evil.com/malicious.js" });
      console.log(">", url.pathname);
      res.end();
      return;
    }

    if (url.pathname === "/redirect-to-non-http") {
      res.writeHead(302, { "Location": "ftp://example.com/file.js" });
      console.log(">", url.pathname);
      res.end();
      return;
    }

    if (url.pathname === "/redirect-chain") {
      res.writeHead(302, { "Location": "/redirect-chain-1" });
      console.log(">", url.pathname);
      res.end();
      return;
    }

    if (url.pathname === "/redirect-chain-1") {
      res.writeHead(302, { "Location": "/redirect-chain-2" });
      console.log(">", url.pathname);
      res.end();
      return;
    }

    if (url.pathname === "/redirect-chain-2") {
      res.writeHead(302, { "Location": "/redirect-chain-3" });
      console.log(">", url.pathname);
      res.end();
      return;
    }

    if (url.pathname === "/redirect-chain-3") {
      res.writeHead(302, { "Location": "/redirect-chain-4" });
      console.log(">", url.pathname);
      res.end();
      return;
    }

    if (url.pathname === "/redirect-chain-4") {
      res.writeHead(302, { "Location": "/redirect-chain-5" });
      console.log(">", url.pathname);
      res.end();
      return;
    }

    if (url.pathname === "/redirect-chain-5") {
      res.writeHead(302, { "Location": "/redirect-chain-6" });
      console.log(">", url.pathname);
      res.end();
      return;
    }

    res.writeHead(404);
    console.log(">", url.pathname, "404");
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
                console.log("sever started")
                resolve();
              }
            });
          });
        }
      }
    );

    compiler.hooks.done.tapAsync("ServerPlugin", (stats, callback) => {
      const s = this.server;
      if (s && --this.refs === 0) {
        this.server = undefined;
        console.log("server closing")
        s.close(callback);
      } else {
        callback();
      }
    });
  }
}

module.exports = ServerPlugin;
