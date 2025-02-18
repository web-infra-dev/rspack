/*
 * MIT License http://www.opensource.org/licenses/mit-license.php
 * Author Tobias Koppers @sokra
 */
import type {
	IncomingMessage,
	ServerOptions as ServerOptionsImport,
	ServerResponse
} from "node:http";
import type { AddressInfo, ListenOptions, Socket } from "node:net";
import type { SecureContextOptions, TlsOptions } from "node:tls";

import type { Compiler, LazyCompilationOptions } from "../..";
import {
	LAZY_COMPILATION_PREFIX,
	lazyCompilationMiddlewareInternal
} from "./middleware";

interface Server {
	on(
		event: "request",
		callback: (req: IncomingMessage, res: ServerResponse) => void
	): void;
	on(event: "connection", callback: (socket: Socket) => void): void;
	on(event: "listening", callback: (err?: Error) => void): void;

	off(
		event: "request",
		callback: (req: IncomingMessage, res: ServerResponse) => void
	): void;

	address(): AddressInfo;
	close(callback: (err?: any) => void): void;
	listen(listenOptions?: number | ListenOptions): void;
}

export interface LazyCompilationDefaultBackendOptions {
	/**
	 * A custom client script path.
	 */
	client?: string;

	/**
	 * Specifies where to listen to from the server.
	 */
	listen?: number | ListenOptions | ((server: Server) => void);

	/**
	 * Specifies the protocol the client should use to connect to the server.
	 */
	protocol?: "http" | "https";

	/**
	 * Specifies how to create the server handling the EventSource requests.
	 */
	server?:
		| ServerOptionsImport<typeof IncomingMessage>
		| ServerOptionsHttps<typeof IncomingMessage, typeof ServerResponse>
		| (() => Server);
}

export type ServerOptionsHttps<
	Request extends typeof IncomingMessage = typeof IncomingMessage,
	Response extends typeof ServerResponse = typeof ServerResponse
> = SecureContextOptions & TlsOptions & ServerOptionsImport<Request, Response>;

const getBackend = (lazyCompilationOptions: LazyCompilationOptions) => {
	const state = {
		module: unimplemented as any,
		dispose: unimplemented as any
	};
	return {
		state,
		backend: (
			compiler: Compiler,
			callback: (
				err: Error | null,
				obj?: {
					dispose: (callback: (err: any) => void) => void;
					module: (args: { module: string; path: string }) => {
						data: string;
						client: string;
						active: boolean;
					};
				}
			) => void
		) => {
			const options = lazyCompilationOptions.backend ?? {};
			const logger = compiler.getInfrastructureLogger("LazyCompilationBackend");
			const activeModules = new Map();
			const filesByKey: Map<string, string> = new Map();
			const isHttps =
				options.protocol === "https" ||
				(typeof options.server === "object" &&
					("key" in options.server || "pfx" in options.server));

			const createServer =
				typeof options.server === "function"
					? options.server
					: (() => {
							const http = isHttps
								? require("node:https")
								: require("node:http");
							return http.createServer.bind(http, options.server);
						})();
			const listen =
				typeof options.listen === "function"
					? options.listen
					: typeof options.server === "function" && !options.listen
						? // if user offers custom server, no need to listen
							() => {}
						: (server: Server) => {
								let { listen } = options;
								if (typeof listen === "object" && !("port" in listen))
									listen = { ...listen, port: undefined };
								server.listen(listen as ListenOptions);
							};

			const protocol = options.protocol || (isHttps ? "https" : "http");
			const middleware = lazyCompilationMiddlewareInternal(
				compiler,
				activeModules,
				filesByKey
			);
			const requestListener = (req: IncomingMessage, res: ServerResponse) => {
				if (!req.url?.startsWith(LAZY_COMPILATION_PREFIX)) {
					// only handle requests that are come from lazyCompilation
					return;
				}

				res.setHeader("Access-Control-Allow-Origin", "*");
				res.setHeader("Access-Control-Allow-Methods", "*");
				res.setHeader("Access-Control-Allow-Headers", "*");
				middleware(req, res, () => {});
			};

			const server = createServer() as Server;
			server.on("request", requestListener);

			let isClosing = false;
			const sockets: Set<Socket> = new Set();
			server.on("connection", socket => {
				sockets.add(socket);
				socket.on("close", () => {
					sockets.delete(socket);
				});
				if (isClosing) socket.destroy();
			});

			server.on("listening", err => {
				if (err) return callback(err);
				const addr = server.address() as AddressInfo;
				if (typeof addr === "string")
					throw new Error("addr must not be a string");
				const urlBase =
					addr.address === "::" || addr.address === "0.0.0.0"
						? `${protocol}://localhost:${addr.port}`
						: addr.family === "IPv6"
							? `${protocol}://[${addr.address}]:${addr.port}`
							: `${protocol}://${addr.address}:${addr.port}`;
				logger.log(
					`Server-Sent-Events server for lazy compilation open at ${urlBase}.`
				);

				state.dispose = (callback: any) => {
					isClosing = true;
					// Removing the listener is a workaround for a memory leak in node.js
					server.off("request", requestListener);
					server.close(err => {
						callback(err);
					});
					for (const socket of sockets) {
						socket.destroy(new Error("Server is disposing"));
					}
				};
				state.module = ({
					module: originalModule,
					path
				}: {
					module: string;
					path: string;
				}) => {
					const key = `${encodeURIComponent(
						originalModule.replace(/\\/g, "/").replace(/@/g, "_")
					).replace(/%(2F|3A|24|26|2B|2C|3B|3D|3A)/g, decodeURIComponent)}`;
					filesByKey.set(key, path);
					const active = activeModules.get(key) > 0;

					return {
						client: `${options.client}?${encodeURIComponent(urlBase + LAZY_COMPILATION_PREFIX)}`,
						data: key,
						active
					};
				};

				callback(null);
			});

			listen(server);
		}
	};
};

export default getBackend;

function unimplemented() {
	throw new Error("access before initialization");
}
