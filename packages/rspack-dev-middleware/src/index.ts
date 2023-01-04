import type { RequestHandler as ExpressRequestHandler } from "express";

export type DevMiddleware = ExpressRequestHandler & {
	invalidate(callback: Function): void;
	close(callback: Function): void;
};

import wdm from "webpack-dev-middleware";

const rdm: typeof wdm = (compiler, options) => {
	if (!options) {
		options = {};
	}
	options.writeToDisk = false;
	return wdm(compiler, options);
};

export default rdm;
export * from "./middleware";
