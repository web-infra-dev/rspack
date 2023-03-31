import { extname } from "path";
import type { IncomingMessage, ServerResponse } from "http";
import type { Compiler } from "@rspack/core";
import wdm from "webpack-dev-middleware";
import type { RequestHandler } from "express";
import mime from "mime-types";
import { parse } from "url";

export function getRspackMemoryAssets(
	compiler: Compiler,
	rdm: ReturnType<typeof wdm>
): RequestHandler {
	const publicPath = compiler.options.output.publicPath
		? compiler.options.output.publicPath.replace(/\/$/, "") + "/"
		: "/";
	return function (
		req: IncomingMessage,
		res: ServerResponse,
		next: () => void
	) {
		const { method, url } = req;
		if (method !== "GET") {
			return next();
		}
		// css hmr will append query string, so here need to remove query string
		const path = parse(url).pathname;
		// asset name is not start with /, so path need to slice 1
		const filename = path.startsWith(publicPath)
			? path.slice(publicPath.length)
			: path.slice(1);
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
		res.end(buffer);
	};
}
