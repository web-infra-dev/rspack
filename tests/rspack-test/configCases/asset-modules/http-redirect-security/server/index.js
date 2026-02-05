const http = require("http");
/**
 * @returns {import("http").Server} server instance
 */
function createServer() {
  const activeConnections = new Set();
  const serverStartTime = Date.now();

  const server = http.createServer((req, res) => {
    const url = new URL(req.url, `http://${req.headers.host}`);
    const reqId = `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
    const relativeTime = Date.now() - serverStartTime;

    console.log(`[${reqId}] [${relativeTime}ms] <`, url.pathname);

    req.on('error', (err) => {
      console.error(`[${reqId}] request error:`, err.message);
    });

    res.on('error', (err) => {
      console.error(`[${reqId}] response error:`, err.message);
    });

    res.on('finish', () => {
      console.log(`[${reqId}] [${Date.now() - serverStartTime}ms] response finished`);
    });

    if (url.pathname === "/redirect-to-disallowed") {
      res.writeHead(302, { "Location": "https://evil.com/malicious.js" });
      console.log(`[${reqId}] [${Date.now() - serverStartTime}ms] >`, url.pathname);
      res.end();
      return;
    }

    if (url.pathname === "/redirect-to-non-http") {
      res.writeHead(302, { "Location": "ftp://example.com/file.js" });
      console.log(`[${reqId}] [${Date.now() - serverStartTime}ms] >`, url.pathname);
      res.end();
      return;
    }

    if (url.pathname === "/redirect-chain") {
      res.writeHead(302, { "Location": "/redirect-chain-1" });
      console.log(`[${reqId}] [${Date.now() - serverStartTime}ms] >`, url.pathname);
      res.end();
      return;
    }

    if (url.pathname === "/redirect-chain-1") {
      res.writeHead(302, { "Location": "/redirect-chain-2" });
      console.log(`[${reqId}] [${Date.now() - serverStartTime}ms] >`, url.pathname);
      res.end();
      return;
    }

    if (url.pathname === "/redirect-chain-2") {
      res.writeHead(302, { "Location": "/redirect-chain-3" });
      console.log(`[${reqId}] [${Date.now() - serverStartTime}ms] >`, url.pathname);
      res.end();
      return;
    }

    if (url.pathname === "/redirect-chain-3") {
      res.writeHead(302, { "Location": "/redirect-chain-4" });
      console.log(`[${reqId}] [${Date.now() - serverStartTime}ms] >`, url.pathname);
      res.end();
      return;
    }

    if (url.pathname === "/redirect-chain-4") {
      res.writeHead(302, { "Location": "/redirect-chain-5" });
      console.log(`[${reqId}] [${Date.now() - serverStartTime}ms] >`, url.pathname);
      res.end();
      return;
    }

    if (url.pathname === "/redirect-chain-5") {
      res.writeHead(302, { "Location": "/redirect-chain-6" });
      console.log(`[${reqId}] [${Date.now() - serverStartTime}ms] >`, url.pathname);
      res.end();
      return;
    }

    res.writeHead(404);
    console.log(`[${reqId}] [${Date.now() - serverStartTime}ms] >`, url.pathname, "404");
    res.end("Not found");
  });

  server.on('connection', (socket) => {
    const connId = `conn-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
    activeConnections.add(socket);
    console.log(`[${connId}] [${Date.now() - serverStartTime}ms] connection opened, total: ${activeConnections.size}`);

    socket.on('close', () => {
      activeConnections.delete(socket);
      console.log(`[${connId}] [${Date.now() - serverStartTime}ms] connection closed, remaining: ${activeConnections.size}`);
    });

    socket.on('error', (err) => {
      console.error(`[${connId}] [${Date.now() - serverStartTime}ms] socket error:`, err.message);
    });
  });

  server.on('error', (err) => {
    console.error('server error:', err.message);
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
    const pluginStartTime = Date.now();

    compiler.hooks.beforeRun.tapPromise(
      "ServerPlugin",
      () => {
        this.refs++;
        console.log(`[SERVER-PLUGIN] beforeRun at ${Date.now() - pluginStartTime}ms, refs: ${this.refs}`);
        if (!this.server) {
          this.server = createServer();
          return new Promise((resolve, reject) => {
            this.server.listen(this.port, err => {
              if (err) {
                reject(err);
              } else {
                console.log(`[SERVER-PLUGIN] server started at ${Date.now() - pluginStartTime}ms`)
                resolve();
              }
            });
          });
        } else {
          return Promise.resolve();
        }
      }
    );

    compiler.hooks.done.tapAsync("ServerPlugin", (stats, callback) => {
      console.log(`[SERVER-PLUGIN] done hook at ${Date.now() - pluginStartTime}ms, current refs before decrement: ${this.refs}`);
      const s = this.server;
      if (s && --this.refs === 0) {
        this.server = undefined;
        console.log(`[SERVER-PLUGIN] server closing at ${Date.now() - pluginStartTime}ms, refs: ${this.refs}`);

        s.close((err) => {
          if (err) {
            console.error(`[SERVER-PLUGIN] server close error at ${Date.now() - pluginStartTime}ms:`, err.message);
          } else {
            console.log(`[SERVER-PLUGIN] server closed successfully at ${Date.now() - pluginStartTime}ms`);
          }
          callback(err);
        });
      } else {
        console.log(`[SERVER-PLUGIN] keeping server alive at ${Date.now() - pluginStartTime}ms, refs: ${this.refs}`);
        callback();
      }
    });
  }
}

module.exports = ServerPlugin;
