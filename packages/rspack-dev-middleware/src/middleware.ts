import { extname, relative } from "path";
import type { Compiler } from "@rspack/core";
import wdm from 'webpack-dev-middleware';
import type { RequestHandler } from "express";
import mime from "mime-types";

export function getRspackMemoryAssets(compiler: Compiler, rdm: ReturnType<typeof wdm>): RequestHandler {
	return function (req, res, next) {
		const { method, path } = req;
		if (method !== "GET") {
			return next();
		}

		const filename = rdm.getFilenameFromUrl(path);
		if (!filename) {
			return next();
		}

		const asset = relative(compiler.outputPath, filename);
		let buffer = compiler.getAsset(asset);
		if (!buffer) {
			return next();
		}

		let contentType =
			mime.contentType(extname(path)) || "text/plain; charset=utf-8";
		res.setHeader("Content-Type", contentType);
		res.send(buffer);
	};
}
