const http = require("http");
const fs = require("fs");
const path = require("path");
const net = require("net");

let globalServer = null;

/**
 * Checks if a port is already in use
 * @param {number} port - The port to check
 * @returns {Promise<boolean>} - True if the port is available, false if in use
 */
function isPortAvailable(port) {
  return new Promise((resolve) => {
    const tester = net.createServer()
      .once('error', () => {
        // Port is in use
        resolve(false);
      })
      .once('listening', () => {
        // Port is available, close the testing server
        tester.close(() => resolve(true));
      })
      .listen(port);
  });
}

/**
 * Creates a simple HTTP server that serves files from the server directory
 * @returns {import("http").Server} server instance
 */
function createServer() {
  const server = http.createServer((req, res) => {
    let file;
    const pathname = "." + req.url.replace(/\?.*$/, "");

    // Add Cache-Control headers for testing caching behavior
    if (req.url.endsWith("?no-cache")) {
      res.setHeader("Cache-Control", "no-cache, no-store");
    } else if (req.url.endsWith("?max-age")) {
      res.setHeader("Cache-Control", "max-age=60");
    } else {
      res.setHeader("Cache-Control", "public, immutable, max-age=600");
    }

    try {
      file = fs
        .readFileSync(path.resolve(__dirname, "server", pathname))
        .toString()
        .replace(/\r\n?/g, "\n")
        .trim();
    } catch (e) {
      // Handle redirects
      if (req.url.includes("redirect")) {
        res.statusCode = 301;
        res.setHeader("Location", "/module.js");
        res.end();
        return;
      }

      // Handle 404
      res.statusCode = 404;
      res.end(`Not found: ${pathname}`);
      return;
    }

    // Set appropriate content type
    res.setHeader(
      "Content-Type",
      pathname.endsWith(".js") ? "text/javascript" :
      pathname.endsWith(".css") ? "text/css" :
      "text/plain"
    );
    res.end(file);
  });

  server.unref();
  return server;
}

class ServerPlugin {
  constructor(port) {
    this.port = port || 8999;
    this.refs = 0;
    this.server = undefined;
  }

  apply(compiler) {
    compiler.hooks.beforeRun.tapPromise(
      "ServerPlugin",
      async () => {
        this.refs++;
        
        // If we already have a global server running, use it
        if (globalServer) {
          this.server = globalServer;
          console.log(`Using existing test server at http://localhost:${this.port}/`);
          return;
        }

        if (!this.server) {
          // Check if port is already in use by another process
          const isAvailable = await isPortAvailable(this.port);
          
          if (!isAvailable) {
            console.log(`Port ${this.port} is already in use, assuming server is running`);
            // Create a dummy server object for tracking references
            this.server = { close: (cb) => cb() };
            return;
          }
          
          this.server = createServer();
          await new Promise((resolve, reject) => {
            this.server.listen(this.port, err => {
              if (err) {
                reject(err);
              } else {
                console.log(`Test server running at http://localhost:${this.port}/`);
                globalServer = this.server; // Store server globally
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
        // Only close if we're the last reference and it's our server
        if (s === globalServer) {
          this.server = undefined;
          globalServer = null;
          s.close(callback);
        } else {
          callback();
        }
      } else {
        callback();
      }
    });
  }
}

exports.ServerPlugin = ServerPlugin; 