const http = require("http");
const fs = require("fs");
const path = require("path");

/**
 * @returns {import("http").Server} server instance
 */
function createServer() {
	const activeConnections = new Set();

	const server = http.createServer((req, res) => {
		const reqId = `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
		console.log(`[${reqId}] request:`, req.url);

		req.on('error', (err) => {
			console.error(`[${reqId}] request error:`, err.message);
		});

		res.on('error', (err) => {
			console.error(`[${reqId}] response error:`, err.message);
		});

		res.on('finish', () => {
			console.log(`[${reqId}] response finished`);
		});

		let file;
		const pathname = "." + req.url.replace(/\?.*$/, "");
		if (req.url.endsWith("?no-cache")) {
			res.setHeader("Cache-Control", "no-cache, max-age=60");
		} else {
			res.setHeader("Cache-Control", "public, immutable, max-age=600");
		}
		try {
			file = fs
				.readFileSync(path.resolve(__dirname, pathname))
				.toString()
				.replace(/\r\n?/g, "\n")
				.trim();
		} catch (e) {
			if (fs.existsSync(path.resolve(__dirname, pathname + ".js"))) {
				res.statusCode = 301;
				res.setHeader("Location", pathname.slice(1) + ".js");
				res.end();
				return;
			}
			res.statusCode = 404;
			res.end();
			return;
		}
		res.setHeader(
			"Content-Type",
			pathname.endsWith(".js") ? "text/javascript" : "text/css"
		);
		res.end(file);
	});

	server.on('connection', (socket) => {
		const connId = `conn-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
		activeConnections.add(socket);
		console.log(`[${connId}] connection opened, total: ${activeConnections.size}`);

		socket.on('close', () => {
			activeConnections.delete(socket);
			console.log(`[${connId}] connection closed, remaining: ${activeConnections.size}`);
		});

		socket.on('error', (err) => {
			console.error(`[${connId}] socket error:`, err.message);
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
		compiler.hooks.beforeRun.tapPromise(
			"ServerPlugin",
			async (compiler, callback) => {
				this.refs++;
				if (!this.server) {
					this.server = createServer();
					await new Promise((resolve, reject) => {
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
			const s = this.server;
			if (s && --this.refs === 0) {
				this.server = undefined;
				console.log("server closing, current refs:", this.refs);

				s.close((err) => {
					if (err) {
						console.error("server close error:", err.message);
					} else {
						console.log("server closed successfully");
					}
					callback(err);
				});
			} else {
				console.log("keeping server alive, current refs:", this.refs);
				callback();
			}
		});
	}
}

module.exports = ServerPlugin;
