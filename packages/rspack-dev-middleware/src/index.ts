import type { Compiler, RspackOptionsNormalized } from "@rspack/core";
import type { RequestHandler as ExpressRequestHandler } from "express";
// import { ready } from "webpack-dev-middleware/dist/utils/ready";

export type RspackDevMiddleware = ExpressRequestHandler & {
	invalidate(callback: Function): void;
	close(callback: Function): void;
};

export function rdm(
	compiler: Compiler,
	options: RspackOptionsNormalized["devServer"]["devMiddleware"],
	// TODO: remove after done hooks completed
	ws: any
): RspackDevMiddleware {
	// @ts-ignore
	const instance: RspackDevMiddleware = wrapper();
	instance.invalidate = function (callback) {
		// TODO:
	};
	instance.close = function (callback) {
		// TODO:
	};

	const watching = compiler.watch({}, ws);
	return instance;
}

function wrapper(): ExpressRequestHandler {
	return function middleware(req, res, next) {
		return next();
	};
}
