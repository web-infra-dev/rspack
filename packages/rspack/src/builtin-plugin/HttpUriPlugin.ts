import path from "node:path";
import { createBrotliDecompress, createGunzip, createInflate } from "node:zlib";
import {
	type BuiltinPlugin,
	BuiltinPluginName,
	type RawHttpUriPluginOptions
} from "@rspack/binding";
import type { Compiler } from "../Compiler";

import type { IncomingMessage } from "node:http";
import { memoize } from "../util/memoize";
import { RspackBuiltinPlugin, createBuiltinPlugin } from "./base";
export type HttpUriPluginOptionsAllowedUris = (string | RegExp)[];

export type HttpUriPluginOptions = {
	/**
	 * A list of allowed URIs
	 */
	allowedUris: HttpUriPluginOptionsAllowedUris;
	/**
	 * Define the location to store the lockfile
	 */
	lockfileLocation?: string;
	/**
	 * Define the location for caching remote resources
	 */
	cacheLocation?: string | false;
	/**
	 * Detect changes to remote resources and upgrade them automatically
	 */
	upgrade?: boolean;
	// /**
	//  * Specify the proxy server to use for fetching remote resources
	//  */
	// proxy?: string;
	// /**
	//  * Freeze the remote resources and lockfile. Any modification to the lockfile or resource contents will result in an error
	//  */
	// frozen?: boolean;
	/**
	 * Custom http client
	 */
	httpClient?: RawHttpUriPluginOptions["httpClient"];
};

const getHttp = memoize(() => require("node:http"));
const getHttps = memoize(() => require("node:https"));
function fetch(url: string, options: { headers: Record<string, string> }) {
	const parsedURL = new URL(url);
	const send: typeof import("node:http") =
		parsedURL.protocol === "https:" ? getHttps() : getHttp();
	return new Promise<{ res: IncomingMessage; body: Buffer }>(
		(resolve, reject) => {
			send
				.get(url, options, res => {
					// align with https://github.com/webpack/webpack/blob/dec18718be5dfba28f067fb3827dd620a1f33667/lib/schemes/HttpUriPlugin.js#L807
					const contentEncoding = res.headers["content-encoding"];
					/** @type {Readable} */
					let stream = res;
					if (contentEncoding === "gzip") {
						stream = stream.pipe(createGunzip()) as any as IncomingMessage;
					} else if (contentEncoding === "br") {
						stream = stream.pipe(
							createBrotliDecompress()
						) as any as IncomingMessage;
					} else if (contentEncoding === "deflate") {
						stream = stream.pipe(createInflate()) as any as IncomingMessage;
					}
					const chunks: Buffer[] = [];
					stream.on("data", chunk => {
						chunks.push(chunk);
					});
					stream.on("end", () => {
						const bodyBuffer = Buffer.concat(chunks);
						if (!res.complete) {
							reject(new Error(`${url} request was terminated early`));
							return;
						}
						resolve({
							res,
							body: bodyBuffer
						});
					});
				})
				.on("error", reject);
		}
	);
}
const defaultHttpClient = async (
	url: string,
	headers: Record<string, string>
) => {
	// Return a promise that resolves to the response
	// setting redirect: "manual" to prevent automatic redirection which will break the redirect logic in rust plugin
	// webpack use require('http').get while rspack use fetch which treats redirect differently
	const { res, body } = await fetch(url, { headers });
	const responseHeaders: Record<string, string> = {};
	for (const [key, value] of Object.entries(res.headers)) {
		if (Array.isArray(value)) {
			responseHeaders[key] = value.join(", ");
		} else {
			responseHeaders[key] = value!;
		}
	}

	// Return the standardized format
	return {
		status: res.statusCode!,
		headers: responseHeaders,
		body: Buffer.from(body)
	};
};

/**
 * Plugin that allows loading modules from HTTP URLs
 */
export class HttpUriPlugin extends RspackBuiltinPlugin {
	name = BuiltinPluginName.HttpUriPlugin;
	affectedHooks = "compilation" as const;

	constructor(private options: HttpUriPluginOptions) {
		super();
	}

	raw(compiler: Compiler): BuiltinPlugin | undefined {
		const { options } = this;
		const lockfileLocation =
			options.lockfileLocation ??
			path.join(
				compiler.context,
				compiler.name ? `${compiler.name}.rspack.lock` : "rspack.lock"
			);
		const cacheLocation =
			options.cacheLocation === false
				? undefined
				: (options.cacheLocation ?? `${lockfileLocation}.data`);

		const raw: RawHttpUriPluginOptions = {
			allowedUris: options.allowedUris,
			lockfileLocation,
			cacheLocation,
			upgrade: options.upgrade ?? false,
			// frozen: options.frozen,
			// proxy: options.proxy,
			httpClient: options.httpClient ?? defaultHttpClient
		};
		return createBuiltinPlugin(this.name, raw);
	}
}
