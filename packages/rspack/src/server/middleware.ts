import * as Connect from "connect";
import * as Sirv from "sirv";
import type { ResolvedRspackOptions } from "../config";

export type MiddlewareServer = Connect.Server;

function createStaticMiddleware(
	options: ResolvedRspackOptions
): Connect.NextHandleFunction {
	const server = Sirv.default(options.dev.static.directory, {
		dev: true
	});
	return function staticMiddleware(req, res, next) {
		server(req, res, next);
	};
}

export function createMiddleware(
	options: ResolvedRspackOptions
): Connect.Server {
	const app = Connect.default();
	app.use(createStaticMiddleware(options));
	return app;
}
