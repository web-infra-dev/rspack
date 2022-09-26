import type { Rspack, ResolvedRspackOptions } from "@rspack/core";
import type { RequestHandler as ExpressRequestHandler } from "express";

export type DevMiddleware = ExpressRequestHandler;

export function rdm(
	compiler: Rspack,
	options: ResolvedRspackOptions["dev"]["devMiddleware"]
): DevMiddleware {
	const instance = wrapper();
	const watching = compiler.watch({});
	return instance;
}

function wrapper(): ExpressRequestHandler {
	return function middleware(req, res, next) {
		console.log("rdm:", req.path);
		return next();
	};
}
