import { extname } from "path";
import type { Compiler } from "@rspack/core";
import type { RequestHandler } from "express";
import mime from "mime-types";

export function getRspackMemoryAssets(compiler: Compiler): RequestHandler {
	return function (req, res, next) {
		const { method, path } = req;
		if (method !== "GET") {
			return next();
		}

		// asset name is not start with /, so path need to slice 1
		let buffer = compiler.getAsset(path.slice(1));
		if (!buffer) {
			return next();
		}

		let contentType =
			mime.contentType(extname(path)) || "text/plain; charset=utf-8";
		res.setHeader("Content-Type", contentType);
		res.send(buffer);
	};
}
