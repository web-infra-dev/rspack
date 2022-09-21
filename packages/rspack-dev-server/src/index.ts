import { Rspack } from "@rspack/core";
import type { Watching, RspackOptions } from "@rspack/core";
import type http from "http";

interface Server {
	close(): Promise<void>;
	start(): Promise<void>;
}

export async function createServer(options: RspackOptions): Promise<Server> {
	const compiler = new Rspack(options);

	const express = (await import("express")).default;
	const app = express();
	app.use(express.static(options.dev.static.directory));

	let watcher: Watching | undefined;
	let server: http.Server | undefined;
	return {
		async close() {
			if (watcher) {
				watcher.close();
			}
			if (server) {
				server.close();
			}
		},
		async start() {
			watcher = await compiler.watch();
			server = app.listen(compiler.options.dev.port, () => {
				console.log(
					`Server listening on http://localhost:${compiler.options.dev.port}`
				);
			});
		}
	};
}
