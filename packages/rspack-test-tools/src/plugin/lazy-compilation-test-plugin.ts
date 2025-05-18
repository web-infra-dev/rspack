import { createServer } from "node:http";
import type { Socket } from "node:net";
import type { AddressInfo } from "node:net";
import { type Compiler, MultiCompiler } from "@rspack/core";
import { experiments } from "@rspack/core";

export class LazyCompilationTestPlugin {
	apply(compiler: Compiler | MultiCompiler) {
		let middleware: any;
		const server = createServer();
		const sockets = new Set<Socket>();

		const promise = new Promise((resolve, reject) => {
			server.on("listening", () => {
				const addr = server.address() as AddressInfo;
				if (typeof addr === "string")
					throw new Error("addr must not be a string");
				const protocol = "http";
				const urlBase =
					addr.address === "::" || addr.address === "0.0.0.0"
						? `${protocol}://localhost:${addr.port}`
						: addr.family === "IPv6"
							? `${protocol}://[${addr.address}]:${addr.port}`
							: `${protocol}://${addr.address}:${addr.port}`;
				if (compiler instanceof MultiCompiler) {
					for (const c of compiler.compilers) {
						if (c.options.experiments.lazyCompilation) {
							c.options.experiments.lazyCompilation.serverUrl = urlBase;
						}
					}
				} else if (compiler.options.experiments.lazyCompilation) {
					compiler.options.experiments.lazyCompilation.serverUrl = urlBase;
				}
				middleware = experiments.lazyCompilationMiddleware(compiler);

				resolve(null);
			});
			server.on("request", (req, res) => {
				middleware(req, res, () => {});
			});
			server.on("connection", socket => {
				sockets.add(socket);
				socket.on("close", () => {
					sockets.delete(socket);
				});
			});
			server.on("error", e => {
				reject(e);
			});
			server.listen();
		});

		let initialized = false;
		compiler.hooks.beforeCompile.tapAsync(
			"LazyCompilationTestPlugin",
			async (_, done) => {
				if (initialized) {
					return done();
				}
				await promise;
				initialized = true;
				done();
			}
		);

		compiler.hooks.shutdown.tapAsync("LazyCompilationTestPlugin", done => {
			server.close(() => {
				done();
			});
			for (const socket of sockets) {
				socket.destroy(new Error("Server is disposing"));
			}
		});
	}
}
