"use strict";

const express = require("express");
const webpack = require("@rspack/core");
const { createProxyMiddleware } = require("http-proxy-middleware");
const { RspackDevServer: Server } = require("@rspack/dev-server");
const config = require("../fixtures/client-config/webpack.config");
const runBrowser = require("../helpers/run-browser");
const [port1, port2] = require("../helpers/ports-map")["allowed-hosts"];

const webSocketServers = ["ws", "sockjs"];

describe("allowed hosts", () => {
  for (const webSocketServer of webSocketServers) {
    it(`should disconnect web socket client using custom hostname from web socket server with the "auto" value based on the "host" header ("${webSocketServer}")`, async () => {
      const devServerHost = "127.0.0.1";
      const devServerPort = port1;
      const proxyHost = devServerHost;
      const proxyPort = port2;

      const compiler = webpack(config);
      const devServerOptions = {
        client: {
          webSocketURL: {
            port: port2
          }
        },
        webSocketServer,
        port: devServerPort,
        host: devServerHost,
        allowedHosts: "auto"
      };
      const server = new Server(devServerOptions, compiler);

      await server.start();

      function startProxy(callback) {
        const app = express();

        app.use(
          "/",
          createProxyMiddleware({
            // Emulation
            onProxyReqWs: proxyReq => {
              proxyReq.setHeader("host", "my-test-host");
            },
            target: `http://${devServerHost}:${devServerPort}`,
            ws: true,
            changeOrigin: true,
            logLevel: "warn"
          })
        );

        return app.listen(proxyPort, proxyHost, callback);
      }

      const proxy = await new Promise(resolve => {
        const proxyCreated = startProxy(() => {
          resolve(proxyCreated);
        });
      });

      const { page, browser } = await runBrowser();

      try {
        const pageErrors = [];
        const consoleMessages = [];

        page
          .on("console", message => {
            consoleMessages.push(message);
          })
          .on("pageerror", error => {
            pageErrors.push(error);
          });

        await page.goto(`http://${proxyHost}:${proxyPort}/`, {
          waitUntil: "networkidle0"
        });

        expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
          "console messages"
        );
        expect(pageErrors).toMatchSnapshot("page errors");
      } catch (error) {
        throw error;
      } finally {
        proxy.close();

        await browser.close();
        await server.stop();
      }
    });

    it(`should disconnect web socket client using custom hostname from web socket server with the "auto" value based on the "host" header when "server: 'https'" is enabled ("${webSocketServer}")`, async () => {
      const devServerHost = "127.0.0.1";
      const devServerPort = port1;
      const proxyHost = devServerHost;
      const proxyPort = port2;

      const compiler = webpack(config);
      const devServerOptions = {
        client: {
          webSocketURL: {
            port: port2,
            protocol: "ws"
          }
        },
        webSocketServer,
        port: devServerPort,
        host: devServerHost,
        allowedHosts: "auto",
        server: "https"
      };
      const server = new Server(devServerOptions, compiler);

      await server.start();

      function startProxy(callback) {
        const app = express();

        app.use(
          "/",
          createProxyMiddleware({
            // Emulation
            onProxyReqWs: proxyReq => {
              proxyReq.setHeader("host", "my-test-host");
            },
            target: `https://${devServerHost}:${devServerPort}`,
            secure: false,
            ws: true,
            changeOrigin: true,
            logLevel: "warn"
          })
        );

        return app.listen(proxyPort, proxyHost, callback);
      }

      const proxy = await new Promise(resolve => {
        const proxyCreated = startProxy(() => {
          resolve(proxyCreated);
        });
      });

      const { page, browser } = await runBrowser();

      try {
        const pageErrors = [];
        const consoleMessages = [];

        page
          .on("console", message => {
            consoleMessages.push(message);
          })
          .on("pageerror", error => {
            pageErrors.push(error);
          });

        await page.goto(`http://${proxyHost}:${proxyPort}/`, {
          waitUntil: "networkidle0"
        });

        expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
          "console messages"
        );
        expect(pageErrors).toMatchSnapshot("page errors");
      } catch (error) {
        throw error;
      } finally {
        proxy.close();

        await browser.close();
        await server.stop();
      }
    });

    it(`should disconnect web socket client using custom hostname from web socket server with the "auto" value based on the "origin" header ("${webSocketServer}")`, async () => {
      const devServerHost = "127.0.0.1";
      const devServerPort = port1;
      const proxyHost = devServerHost;
      const proxyPort = port2;

      const compiler = webpack(config);
      const devServerOptions = {
        client: {
          webSocketURL: {
            port: port2
          }
        },
        webSocketServer,
        port: devServerPort,
        host: devServerHost,
        allowedHosts: "auto"
      };
      const server = new Server(devServerOptions, compiler);

      await server.start();

      function startProxy(callback) {
        const app = express();

        app.use(
          "/",
          createProxyMiddleware({
            // Emulation
            onProxyReqWs: proxyReq => {
              proxyReq.setHeader("origin", "http://my-test-origin.com/");
            },
            target: `http://${devServerHost}:${devServerPort}`,
            ws: true,
            changeOrigin: true,
            logLevel: "warn"
          })
        );

        return app.listen(proxyPort, proxyHost, callback);
      }

      const proxy = await new Promise(resolve => {
        const proxyCreated = startProxy(() => {
          resolve(proxyCreated);
        });
      });

      const { page, browser } = await runBrowser();

      try {
        const pageErrors = [];
        const consoleMessages = [];

        page
          .on("console", message => {
            consoleMessages.push(message);
          })
          .on("pageerror", error => {
            pageErrors.push(error);
          });

        await page.goto(`http://${proxyHost}:${proxyPort}/`, {
          waitUntil: "networkidle0"
        });

        expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
          "console messages"
        );
        expect(pageErrors).toMatchSnapshot("page errors");
      } catch (error) {
        throw error;
      } finally {
        proxy.close();

        await browser.close();
        await server.stop();
      }
    });

    it(`should connect web socket client using localhost to web socket server with the "auto" value ("${webSocketServer}")`, async () => {
      const devServerHost = "localhost";
      const devServerPort = port1;
      const proxyHost = devServerHost;
      const proxyPort = port2;

      const compiler = webpack(config);
      const devServerOptions = {
        client: {
          webSocketURL: {
            port: port2
          }
        },
        webSocketServer,
        port: devServerPort,
        host: devServerHost,
        allowedHosts: "auto"
      };
      const server = new Server(devServerOptions, compiler);

      await server.start();

      function startProxy(callback) {
        const app = express();

        app.use(
          "/",
          createProxyMiddleware({
            target: `http://${devServerHost}:${devServerPort}`,
            ws: true,
            changeOrigin: true,
            logLevel: "warn"
          })
        );

        return app.listen(proxyPort, proxyHost, callback);
      }

      const proxy = await new Promise(resolve => {
        const proxyCreated = startProxy(() => {
          resolve(proxyCreated);
        });
      });

      const { page, browser } = await runBrowser();
      try {
        const pageErrors = [];
        const consoleMessages = [];

        page
          .on("console", message => {
            consoleMessages.push(message);
          })
          .on("pageerror", error => {
            pageErrors.push(error);
          });

        await page.goto(`http://${proxyHost}:${proxyPort}/`, {
          waitUntil: "networkidle0"
        });

        expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
          "console messages"
        );
        expect(pageErrors).toMatchSnapshot("page errors");
      } catch (error) {
        throw error;
      } finally {
        proxy.close();

        await browser.close();
        await server.stop();
      }
    });

    it(`should connect web socket client using "127.0.0.1" host to web socket server with the "auto" value ("${webSocketServer}")`, async () => {
      const devServerHost = "127.0.0.1";
      const devServerPort = port1;
      const proxyHost = devServerHost;
      const proxyPort = port2;

      const compiler = webpack(config);
      const devServerOptions = {
        client: {
          webSocketURL: {
            port: port2
          }
        },
        webSocketServer,
        port: devServerPort,
        host: devServerHost,
        allowedHosts: "auto"
      };
      const server = new Server(devServerOptions, compiler);

      await server.start();

      function startProxy(callback) {
        const app = express();

        app.use(
          "/",
          createProxyMiddleware({
            target: `http://${devServerHost}:${devServerPort}`,
            ws: true,
            changeOrigin: true,
            logLevel: "warn"
          })
        );

        return app.listen(proxyPort, proxyHost, callback);
      }

      const proxy = await new Promise(resolve => {
        const proxyCreated = startProxy(() => {
          resolve(proxyCreated);
        });
      });

      const { page, browser } = await runBrowser();

      try {
        const pageErrors = [];
        const consoleMessages = [];

        page
          .on("console", message => {
            consoleMessages.push(message);
          })
          .on("pageerror", error => {
            pageErrors.push(error);
          });

        await page.goto(`http://${proxyHost}:${proxyPort}/`, {
          waitUntil: "networkidle0"
        });

        expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
          "console messages"
        );
        expect(pageErrors).toMatchSnapshot("page errors");
      } catch (error) {
        throw error;
      } finally {
        proxy.close();

        await browser.close();
        await server.stop();
      }
    });

    it(`should connect web socket client using "[::1] host to web socket server with the "auto" value ("${webSocketServer}")`, async () => {
      const devServerHost = "::1";
      const devServerPort = port1;
      const proxyHost = devServerHost;
      const proxyPort = port2;

      const compiler = webpack(config);
      const devServerOptions = {
        client: {
          webSocketURL: {
            port: port2
          }
        },
        webSocketServer,
        port: devServerPort,
        host: devServerHost,
        allowedHosts: "auto"
      };
      const server = new Server(devServerOptions, compiler);

      await server.start();

      function startProxy(callback) {
        const app = express();

        app.use(
          "/",
          createProxyMiddleware({
            target: `http://[${devServerHost}]:${devServerPort}`,
            ws: true,
            changeOrigin: true,
            logLevel: "warn"
          })
        );

        return app.listen(proxyPort, proxyHost, callback);
      }

      const proxy = await new Promise(resolve => {
        const proxyCreated = startProxy(() => {
          resolve(proxyCreated);
        });
      });

      const { page, browser } = await runBrowser();

      try {
        const pageErrors = [];
        const consoleMessages = [];

        page
          .on("console", message => {
            consoleMessages.push(message);
          })
          .on("pageerror", error => {
            pageErrors.push(error);
          });

        await page.goto(`http://[${proxyHost}]:${proxyPort}/`, {
          waitUntil: "networkidle0"
        });

        expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
          "console messages"
        );
        expect(pageErrors).toMatchSnapshot("page errors");
      } catch (error) {
        throw error;
      } finally {
        proxy.close();

        await browser.close();
        await server.stop();
      }
    });

    it(`should connect web socket client using "file:" protocol to web socket server with the "auto" value ("${webSocketServer}")`, async () => {
      const devServerHost = "127.0.0.1";
      const devServerPort = port1;
      const proxyHost = devServerHost;
      const proxyPort = port2;

      const compiler = webpack(config);
      const devServerOptions = {
        client: {
          webSocketURL: {
            port: port2
          }
        },
        webSocketServer,
        port: devServerPort,
        host: devServerHost,
        allowedHosts: "auto"
      };
      const server = new Server(devServerOptions, compiler);

      await server.start();

      function startProxy(callback) {
        const app = express();

        app.use(
          "/",
          createProxyMiddleware({
            target: `http://${devServerHost}:${devServerPort}`,
            onProxyReqWs: proxyReq => {
              proxyReq.setHeader("origin", "file:///path/to/local/file.js");
            },
            ws: true,
            changeOrigin: true,
            logLevel: "warn"
          })
        );

        return app.listen(proxyPort, proxyHost, callback);
      }

      const proxy = await new Promise(resolve => {
        const proxyCreated = startProxy(() => {
          resolve(proxyCreated);
        });
      });

      const { page, browser } = await runBrowser();

      try {
        const pageErrors = [];
        const consoleMessages = [];

        page
          .on("console", message => {
            consoleMessages.push(message);
          })
          .on("pageerror", error => {
            pageErrors.push(error);
          });

        await page.goto(`http://${proxyHost}:${proxyPort}/`, {
          waitUntil: "networkidle0"
        });

        expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
          "console messages"
        );
        expect(pageErrors).toMatchSnapshot("page errors");
      } catch (error) {
        throw error;
      } finally {
        proxy.close();

        await browser.close();
        await server.stop();
      }
    });

    it(`should connect web socket client using "chrome-extension:" protocol to web socket server with the "auto" value ("${webSocketServer}")`, async () => {
      const devServerHost = "127.0.0.1";
      const devServerPort = port1;
      const proxyHost = devServerHost;
      const proxyPort = port2;

      const compiler = webpack(config);
      const devServerOptions = {
        client: {
          webSocketURL: {
            port: port2
          }
        },
        webSocketServer,
        port: devServerPort,
        host: devServerHost,
        allowedHosts: "auto"
      };
      const server = new Server(devServerOptions, compiler);

      await server.start();

      function startProxy(callback) {
        const app = express();

        app.use(
          "/",
          createProxyMiddleware({
            target: `http://${devServerHost}:${devServerPort}`,
            onProxyReqWs: proxyReq => {
              proxyReq.setHeader("origin", "chrome-extension:///abcdef");
            },
            ws: true,
            changeOrigin: true,
            logLevel: "warn"
          })
        );

        return app.listen(proxyPort, proxyHost, callback);
      }

      const proxy = await new Promise(resolve => {
        const proxyCreated = startProxy(() => {
          resolve(proxyCreated);
        });
      });

      const { page, browser } = await runBrowser();

      try {
        const pageErrors = [];
        const consoleMessages = [];

        page
          .on("console", message => {
            consoleMessages.push(message);
          })
          .on("pageerror", error => {
            pageErrors.push(error);
          });

        await page.goto(`http://${proxyHost}:${proxyPort}/`, {
          waitUntil: "networkidle0"
        });

        expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
          "console messages"
        );
        expect(pageErrors).toMatchSnapshot("page errors");
      } catch (error) {
        throw error;
      } finally {
        proxy.close();

        await browser.close();
        await server.stop();
      }
    });

    it(`should connect web socket client using custom hostname to web socket server with the "all" value ("${webSocketServer}")`, async () => {
      const devServerHost = "127.0.0.1";
      const devServerPort = port1;
      const proxyHost = devServerHost;
      const proxyPort = port2;

      const compiler = webpack(config);
      const devServerOptions = {
        client: {
          webSocketURL: {
            port: port2
          }
        },
        webSocketServer,
        port: devServerPort,
        host: devServerHost,
        allowedHosts: "all"
      };
      const server = new Server(devServerOptions, compiler);

      await server.start();

      function startProxy(callback) {
        const app = express();

        app.use(
          "/",
          createProxyMiddleware({
            // Emulation
            onProxyReqWs: proxyReq => {
              proxyReq.setHeader("origin", "http://my-test-origin.com/");
            },
            target: `http://${devServerHost}:${devServerPort}`,
            ws: true,
            changeOrigin: true,
            logLevel: "warn"
          })
        );

        return app.listen(proxyPort, proxyHost, callback);
      }

      const proxy = await new Promise(resolve => {
        const proxyCreated = startProxy(() => {
          resolve(proxyCreated);
        });
      });

      const { page, browser } = await runBrowser();

      try {
        const pageErrors = [];
        const consoleMessages = [];

        page
          .on("console", message => {
            consoleMessages.push(message);
          })
          .on("pageerror", error => {
            pageErrors.push(error);
          });

        await page.goto(`http://${proxyHost}:${proxyPort}/`, {
          waitUntil: "networkidle0"
        });

        expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
          "console messages"
        );
        expect(pageErrors).toMatchSnapshot("page errors");
      } catch (error) {
        throw error;
      } finally {
        proxy.close();

        await browser.close();
        await server.stop();
      }
    });

    it(`should connect web socket client using custom hostname to web socket server with the "all" value in array ("${webSocketServer}")`, async () => {
      const devServerHost = "127.0.0.1";
      const devServerPort = port1;
      const proxyHost = devServerHost;
      const proxyPort = port2;

      const compiler = webpack(config);
      const devServerOptions = {
        client: {
          webSocketURL: {
            port: port2
          }
        },
        webSocketServer,
        port: devServerPort,
        host: devServerHost,
        allowedHosts: ["all"]
      };
      const server = new Server(devServerOptions, compiler);

      await server.start();

      function startProxy(callback) {
        const app = express();

        app.use(
          "/",
          createProxyMiddleware({
            // Emulation
            onProxyReqWs: proxyReq => {
              proxyReq.setHeader("origin", "http://my-test-origin.com/");
            },
            target: `http://${devServerHost}:${devServerPort}`,
            ws: true,
            changeOrigin: true,
            logLevel: "warn"
          })
        );

        return app.listen(proxyPort, proxyHost, callback);
      }

      const proxy = await new Promise(resolve => {
        const proxyCreated = startProxy(() => {
          resolve(proxyCreated);
        });
      });

      const { page, browser } = await runBrowser();

      try {
        const pageErrors = [];
        const consoleMessages = [];

        page
          .on("console", message => {
            consoleMessages.push(message);
          })
          .on("pageerror", error => {
            pageErrors.push(error);
          });

        await page.goto(`http://${proxyHost}:${proxyPort}/`, {
          waitUntil: "networkidle0"
        });

        expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
          "console messages"
        );
        expect(pageErrors).toMatchSnapshot("page errors");
      } catch (error) {
        throw error;
      } finally {
        proxy.close();

        await browser.close();
        await server.stop();
      }
    });

    it(`should connect web socket client using custom hostname to web socket server with the custom hostname value ("${webSocketServer}")`, async () => {
      const devServerHost = "127.0.0.1";
      const devServerPort = port1;
      const proxyHost = devServerHost;
      const proxyPort = port2;

      const compiler = webpack(config);
      const devServerOptions = {
        client: {
          webSocketURL: {
            port: port2
          }
        },
        webSocketServer,
        port: devServerPort,
        host: devServerHost,
        allowedHosts: "my-test-origin.com"
      };
      const server = new Server(devServerOptions, compiler);

      await server.start();

      function startProxy(callback) {
        const app = express();

        app.use(
          "/",
          createProxyMiddleware({
            // Emulation
            onProxyReqWs: proxyReq => {
              proxyReq.setHeader("origin", "http://my-test-origin.com/");
            },
            target: `http://${devServerHost}:${devServerPort}`,
            ws: true,
            changeOrigin: true,
            logLevel: "warn"
          })
        );

        return app.listen(proxyPort, proxyHost, callback);
      }

      const proxy = await new Promise(resolve => {
        const proxyCreated = startProxy(() => {
          resolve(proxyCreated);
        });
      });

      const { page, browser } = await runBrowser();

      try {
        const pageErrors = [];
        const consoleMessages = [];

        page
          .on("console", message => {
            consoleMessages.push(message);
          })
          .on("pageerror", error => {
            pageErrors.push(error);
          });

        await page.goto(`http://${proxyHost}:${proxyPort}/`, {
          waitUntil: "networkidle0"
        });

        expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
          "console messages"
        );
        expect(pageErrors).toMatchSnapshot("page errors");
      } catch (error) {
        throw error;
      } finally {
        proxy.close();

        await browser.close();
        await server.stop();
      }
    });

    it(`should connect web socket client using custom hostname to web socket server with the custom hostname value starting with dot ("${webSocketServer}")`, async () => {
      const devServerHost = "127.0.0.1";
      const devServerPort = port1;
      const proxyHost = devServerHost;
      const proxyPort = port2;

      const compiler = webpack(config);
      const devServerOptions = {
        client: {
          webSocketURL: {
            port: port2
          }
        },
        webSocketServer,
        port: devServerPort,
        host: devServerHost,
        allowedHosts: ".my-test-origin.com"
      };
      const server = new Server(devServerOptions, compiler);

      await server.start();

      function startProxy(callback) {
        const app = express();

        app.use(
          "/",
          createProxyMiddleware({
            // Emulation
            onProxyReqWs: proxyReq => {
              proxyReq.setHeader("origin", "http://my-test-origin.com/");
            },
            target: `http://${devServerHost}:${devServerPort}`,
            ws: true,
            changeOrigin: true,
            logLevel: "warn"
          })
        );

        return app.listen(proxyPort, proxyHost, callback);
      }

      const proxy = await new Promise(resolve => {
        const proxyCreated = startProxy(() => {
          resolve(proxyCreated);
        });
      });

      const { page, browser } = await runBrowser();

      try {
        const pageErrors = [];
        const consoleMessages = [];

        page
          .on("console", message => {
            consoleMessages.push(message);
          })
          .on("pageerror", error => {
            pageErrors.push(error);
          });

        await page.goto(`http://${proxyHost}:${proxyPort}/`, {
          waitUntil: "networkidle0"
        });

        expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
          "console messages"
        );
        expect(pageErrors).toMatchSnapshot("page errors");
      } catch (error) {
        throw error;
      } finally {
        proxy.close();

        await browser.close();
        await server.stop();
      }
    });

    it(`should connect web socket client using custom sub hostname to web socket server with the custom hostname value ("${webSocketServer}")`, async () => {
      const devServerHost = "127.0.0.1";
      const devServerPort = port1;
      const proxyHost = devServerHost;
      const proxyPort = port2;

      const compiler = webpack(config);
      const devServerOptions = {
        client: {
          webSocketURL: {
            port: port2
          }
        },
        webSocketServer,
        port: devServerPort,
        host: devServerHost,
        allowedHosts: ".my-test-origin.com"
      };
      const server = new Server(devServerOptions, compiler);

      await server.start();

      function startProxy(callback) {
        const app = express();

        app.use(
          "/",
          createProxyMiddleware({
            // Emulation
            onProxyReqWs: proxyReq => {
              proxyReq.setHeader(
                "origin",
                "http://foo.bar.baz.my-test-origin.com/"
              );
            },
            target: `http://${devServerHost}:${devServerPort}`,
            ws: true,
            changeOrigin: true,
            logLevel: "warn"
          })
        );

        return app.listen(proxyPort, proxyHost, callback);
      }

      const proxy = await new Promise(resolve => {
        const proxyCreated = startProxy(() => {
          resolve(proxyCreated);
        });
      });

      const { page, browser } = await runBrowser();

      try {
        const pageErrors = [];
        const consoleMessages = [];

        page
          .on("console", message => {
            consoleMessages.push(message);
          })
          .on("pageerror", error => {
            pageErrors.push(error);
          });

        await page.goto(`http://${proxyHost}:${proxyPort}/`, {
          waitUntil: "networkidle0"
        });

        expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
          "console messages"
        );
        expect(pageErrors).toMatchSnapshot("page errors");
      } catch (error) {
        throw error;
      } finally {
        proxy.close();

        await browser.close();
        await server.stop();
      }
    });

    it(`should connect web socket client using custom hostname to web socket server with the multiple custom hostname values ("${webSocketServer}")`, async () => {
      const devServerHost = "127.0.0.1";
      const devServerPort = port1;
      const proxyHost = devServerHost;
      const proxyPort = port2;

      const compiler = webpack(config);
      const devServerOptions = {
        client: {
          webSocketURL: {
            port: port2
          }
        },
        webSocketServer,
        port: devServerPort,
        host: devServerHost,
        allowedHosts: ["my-test-origin.com"]
      };
      const server = new Server(devServerOptions, compiler);

      await server.start();

      function startProxy(callback) {
        const app = express();

        app.use(
          "/",
          createProxyMiddleware({
            // Emulation
            onProxyReqWs: proxyReq => {
              proxyReq.setHeader("origin", "http://my-test-origin.com/");
            },
            target: `http://${devServerHost}:${devServerPort}`,
            ws: true,
            changeOrigin: true,
            logLevel: "warn"
          })
        );

        return app.listen(proxyPort, proxyHost, callback);
      }

      const proxy = await new Promise(resolve => {
        const proxyCreated = startProxy(() => {
          resolve(proxyCreated);
        });
      });

      const { page, browser } = await runBrowser();

      try {
        const pageErrors = [];
        const consoleMessages = [];

        page
          .on("console", message => {
            consoleMessages.push(message);
          })
          .on("pageerror", error => {
            pageErrors.push(error);
          });

        await page.goto(`http://${proxyHost}:${proxyPort}/`, {
          waitUntil: "networkidle0"
        });

        expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
          "console messages"
        );
        expect(pageErrors).toMatchSnapshot("page errors");
      } catch (error) {
        throw error;
      } finally {
        proxy.close();

        await browser.close();
        await server.stop();
      }
    });

    it(`should disconnect web client using localhost to web socket server with the "auto" value ("${webSocketServer}")`, async () => {
      const devServerHost = "127.0.0.1";
      const devServerPort = port1;
      const proxyHost = devServerHost;
      const proxyPort = port2;

      const compiler = webpack(config);
      const devServerOptions = {
        client: {
          webSocketURL: {
            port: port2
          }
        },
        webSocketServer,
        port: devServerPort,
        host: devServerHost,
        allowedHosts: "auto"
      };
      const server = new Server(devServerOptions, compiler);

      await server.start();

      function startProxy(callback) {
        const app = express();

        app.use(
          "/",
          createProxyMiddleware({
            // Emulation
            onProxyReq: (proxyReq, req, res) => {
              proxyReq.setHeader("host", "unknown");
              res.setHeader("host", devServerHost);
            },
            target: `http://${devServerHost}:${devServerPort}`,
            ws: true,
            changeOrigin: true,
            logLevel: "warn"
          })
        );

        return app.listen(proxyPort, proxyHost, callback);
      }

      const proxy = await new Promise(resolve => {
        const proxyCreated = startProxy(() => {
          resolve(proxyCreated);
        });
      });

      const { page, browser } = await runBrowser();

      try {
        const pageErrors = [];
        const consoleMessages = [];

        page
          .on("console", message => {
            consoleMessages.push(message);
          })
          .on("pageerror", error => {
            pageErrors.push(error);
          });

        await page.goto(`http://${proxyHost}:${proxyPort}/`, {
          waitUntil: "networkidle0"
        });

        const html = await page.content();

        expect(html).toMatchSnapshot("html");
        expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
          "console messages"
        );
        expect(pageErrors).toMatchSnapshot("page errors");
      } catch (error) {
        throw error;
      } finally {
        proxy.close();

        await browser.close();
        await server.stop();
      }
    });
  }

  describe("check host headers", () => {
    let compiler;
    let server;
    let page;
    let browser;
    let pageErrors;
    let consoleMessages;

    beforeEach(() => {
      compiler = webpack(config);
      pageErrors = [];
      consoleMessages = [];
    });

    afterEach(async () => {
      await browser.close();
      await server.stop();
    });

    it("should always allow `localhost` if options.allowedHosts is auto", async () => {
      const options = {
        allowedHosts: "auto",
        port: port1
      };

      const headers = {
        host: "localhost"
      };

      server = new Server(options, compiler);

      await server.start();

      ({ page, browser } = await runBrowser());

      page
        .on("console", message => {
          consoleMessages.push(message);
        })
        .on("pageerror", error => {
          pageErrors.push(error);
        });

      const response = await page.goto(`http://127.0.0.1:${port1}/main.js`, {
        waitUntil: "networkidle0"
      });

      if (!server.checkHeader(headers, "host")) {
        throw new Error("Validation didn't fail");
      }

      expect(response.status()).toMatchSnapshot("response status");

      expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
        "console messages"
      );

      expect(pageErrors).toMatchSnapshot("page errors");
    });

    it("should always allow `localhost` subdomain if options.allowedHosts is auto", async () => {
      const options = {
        allowedHosts: "auto",
        port: port1
      };

      const headers = {
        host: "app.localhost"
      };

      server = new Server(options, compiler);

      await server.start();

      ({ page, browser } = await runBrowser());

      page
        .on("console", message => {
          consoleMessages.push(message);
        })
        .on("pageerror", error => {
          pageErrors.push(error);
        });

      const response = await page.goto(`http://127.0.0.1:${port1}/main.js`, {
        waitUntil: "networkidle0"
      });

      if (!server.checkHeader(headers, "host")) {
        throw new Error("Validation didn't fail");
      }

      expect(response.status()).toMatchSnapshot("response status");

      expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
        "console messages"
      );

      expect(pageErrors).toMatchSnapshot("page errors");
    });

    it("should always allow value from the `host` options if options.allowedHosts is auto", async () => {
      const networkIP = Server.internalIPSync("v4");
      const options = {
        host: networkIP,
        allowedHosts: "auto",
        port: port1
      };

      const headers = {
        host: networkIP
      };

      server = new Server(options, compiler);

      await server.start();

      ({ page, browser } = await runBrowser());

      page
        .on("console", message => {
          consoleMessages.push(message);
        })
        .on("pageerror", error => {
          pageErrors.push(error);
        });

      const response = await page.goto(`http://${networkIP}:${port1}/main.js`, {
        waitUntil: "networkidle0"
      });

      if (!server.checkHeader(headers, "host")) {
        throw new Error("Validation didn't fail");
      }

      expect(response.status()).toMatchSnapshot("response status");

      expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
        "console messages"
      );

      expect(pageErrors).toMatchSnapshot("page errors");
    });

    it("should always allow value of the `host` option from the `client.webSocketURL` option if options.allowedHosts is auto", async () => {
      const options = {
        allowedHosts: "auto",
        port: port1,
        client: {
          webSocketURL: "ws://test.host:80"
        }
      };

      const headers = {
        host: "test.host"
      };

      server = new Server(options, compiler);

      await server.start();

      ({ page, browser } = await runBrowser());

      page
        .on("console", message => {
          consoleMessages.push(message);
        })
        .on("pageerror", error => {
          pageErrors.push(error);
        });

      const response = await page.goto(`http://127.0.0.1:${port1}/main.js`, {
        waitUntil: "networkidle0"
      });

      if (!server.checkHeader(headers, "host")) {
        throw new Error("Validation didn't fail");
      }

      expect(response.status()).toMatchSnapshot("response status");

      expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
        "console messages"
      );

      expect(pageErrors).toMatchSnapshot("page errors");
    });

    it("should always allow any host if options.allowedHosts is all", async () => {
      const options = {
        allowedHosts: "all",
        port: port1
      };
      const headers = {
        host: "bad.host"
      };

      server = new Server(options, compiler);

      await server.start();

      ({ page, browser } = await runBrowser());

      page
        .on("console", message => {
          consoleMessages.push(message);
        })
        .on("pageerror", error => {
          pageErrors.push(error);
        });

      const response = await page.goto(`http://127.0.0.1:${port1}/main.js`, {
        waitUntil: "networkidle0"
      });

      if (!server.checkHeader(headers, "host")) {
        throw new Error("Validation didn't fail");
      }

      expect(response.status()).toMatchSnapshot("response status");

      expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
        "console messages"
      );

      expect(pageErrors).toMatchSnapshot("page errors");
    });

    it("should allow hosts in allowedHosts", async () => {
      const tests = ["test.host", "test2.host", "test3.host"];
      const options = {
        allowedHosts: tests,
        port: port1
      };

      server = new Server(options, compiler);

      await server.start();

      ({ page, browser } = await runBrowser());

      page
        .on("console", message => {
          consoleMessages.push(message);
        })
        .on("pageerror", error => {
          pageErrors.push(error);
        });

      const response = await page.goto(`http://127.0.0.1:${port1}/main.js`, {
        waitUntil: "networkidle0"
      });

      tests.forEach(test => {
        const headers = { host: test };

        if (!server.checkHeader(headers, "host")) {
          throw new Error("Validation didn't fail");
        }
      });

      expect(response.status()).toMatchSnapshot("response status");

      expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
        "console messages"
      );

      expect(pageErrors).toMatchSnapshot("page errors");
    });

    it("should allow hosts that pass a wildcard in allowedHosts", async () => {
      const options = {
        allowedHosts: [".example.com"],
        port: port1
      };

      server = new Server(options, compiler);

      await server.start();

      ({ page, browser } = await runBrowser());

      page
        .on("console", message => {
          consoleMessages.push(message);
        })
        .on("pageerror", error => {
          pageErrors.push(error);
        });

      const response = await page.goto(`http://127.0.0.1:${port1}/main.js`, {
        waitUntil: "networkidle0"
      });

      const tests = [
        "www.example.com",
        "subdomain.example.com",
        "example.com",
        "subsubcomain.subdomain.example.com",
        "example.com:80",
        "subdomain.example.com:80"
      ];

      tests.forEach(test => {
        const headers = { host: test };

        if (!server.checkHeader(headers, "host")) {
          throw new Error("Validation didn't fail");
        }
      });

      expect(response.status()).toMatchSnapshot("response status");

      expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
        "console messages"
      );

      expect(pageErrors).toMatchSnapshot("page errors");
    });
  });
});
