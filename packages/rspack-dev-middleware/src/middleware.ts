import { extname, relative } from "path";
import type { Compiler } from "@rspack/core";
import wdm from "webpack-dev-middleware";
import type { RequestHandler } from "express";
import mime from "mime-types";

export function getRspackMemoryAssets(
	compiler: Compiler,
	rdm: ReturnType<typeof wdm>
): RequestHandler {
	return function (req, res, next) {
		const { method, path } = req;
		if (method !== "GET") {
			return next();
		}

		// asset name is not start with /, so path need to slice 1
		const filename = path.slice(1);
		let buffer =
			compiler.getAsset(filename) ??
			(() => {
				const { index } = rdm.context.options;
				const indexValue =
					typeof index === "undefined" || typeof index === "boolean"
						? "index.html"
						: index;
				return compiler.getAsset(filename + indexValue);
			})();
		if (!buffer) {
			return next();
		}
		let contentType;
		if (filename === "") {
			contentType = "text/html; charset=utf-8";
		} else {
			contentType =
				mime.contentType(extname(path)) || "text/plain; charset=utf-8";
		}

		res.setHeader("Content-Type", contentType);
		res.send(buffer);
	};
}
