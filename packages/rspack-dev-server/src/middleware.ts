import crypto from "node:crypto";
import type { IncomingMessage } from "node:http";
import { extname } from "node:path";
import { parse } from "node:url";
import type { Compilation, Compiler } from "@rspack/core";
import type { RequestHandler, Response } from "express";
import mime from "mime-types";
import type wdm from "webpack-dev-middleware";

function etag(buf: any) {
	const hash = crypto.createHash("sha256").update(buf).digest("hex");
	const etag = hash;
	return etag;
}

function createPublicPathGetter(compiler: Compiler) {
	const raw = compiler.options.output.publicPath || "/";

	if (typeof raw === "function") {
		return (compilation?: Compilation) =>
			compilation ? compilation.getPath(raw) : raw({ hash: "XXXX" }, undefined);
	}
	if (/\[(hash|fullhash)[:\]]/.test(raw)) {
		return (compilation?: Compilation) =>
			compilation ? compilation.getPath(raw) : `${raw.replace(/\/$/, "")}/`;
	}
	return () => `${raw.replace(/\/$/, "")}/`;
}

export function getRspackMemoryAssets(
	compiler: Compiler,
	rdm: ReturnType<typeof wdm>
): RequestHandler {
	const getPublicPath = createPublicPathGetter(compiler);

	return (req: IncomingMessage, res: Response, next: () => void) => {
		const { method, url } = req;
		if (method !== "GET") {
			return next();
		}

		// css hmr will append query string, so here need to remove query string
		// @ts-expect-error
		const path = parse(url).pathname;
		const publicPath = getPublicPath(compiler._lastCompilation);
		// asset name is not start with /, so path need to slice 1
		// @ts-expect-error
		const filename = path.startsWith(publicPath)
			? // @ts-expect-error
				path.slice(publicPath.length)
			: // @ts-expect-error
				path.slice(1);
		const buffer =
			compiler._lastCompilation?.getAsset(filename) ??
			(() => {
				const { index } = rdm.context.options;
				const indexValue =
					typeof index === "undefined" || typeof index === "boolean"
						? "index.html"
						: index;
				return compiler._lastCompilation?.getAsset(filename + indexValue);
			})();
		if (!buffer) {
			return next();
		}
		let contentType: string;
		if (filename === "") {
			contentType = "text/html; charset=utf-8";
		} else {
			contentType =
				// @ts-expect-error
				mime.contentType(extname(path)) || "text/plain; charset=utf-8";
		}

		const calcEtag = etag(buffer);

		const oldEtag = req.headers["if-none-match"];
		res.setHeader("Content-Type", contentType);
		res.setHeader("ETag", calcEtag);
		if (calcEtag === oldEtag) {
			res.status(304).send();
		} else {
			res.send(buffer);
		}
	};
}
