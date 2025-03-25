import {
	BuiltinPluginName,
	type RawHttpUriPluginOptions
} from "@rspack/binding";
import { create } from "./base";

export type HttpUriPluginOptions = {
	/**
	 * The allowed URIs regexp
	 */
	allowedUris?: (string | RegExp)[];
	/**
	 * The cache location for HTTP responses
	 */
	cacheLocation?: string | false;
	/**
	 * Whether to freeze the HTTP cache
	 */
	frozen?: boolean;
	/**
	 * The lockfile location
	 */
	lockfileLocation?: string;
	/**
	 * The proxy to use for HTTP requests
	 */
	proxy?: string;
	/**
	 * Whether to upgrade dependencies
	 */
	upgrade?: boolean;
};

/**
 * Plugin that allows loading modules from HTTP URLs
 */
export const HttpUriPlugin = create(
	BuiltinPluginName.HttpUriPlugin,
	(options: HttpUriPluginOptions = {}): RawHttpUriPluginOptions => {
		const httpClient = (url: string, headers: Record<string, string>) => {
			// Return a promise that resolves to the response
			return fetch(url, { headers }).then(response => {
				// Convert the response to the format expected by the HTTP client
				return response.arrayBuffer().then(buffer => {
					// Extract headers
					const responseHeaders: Record<string, string> = {};
					response.headers.forEach((value, key) => {
						responseHeaders[key] = value;
					});

					// Return the standardized format
					return {
						status: response.status,
						headers: responseHeaders,
						body: Buffer.from(buffer)
					};
				});
			});
		};

		return {
			allowedUris: options.allowedUris,
			cacheLocation: options.cacheLocation || undefined,
			frozen: options.frozen,
			lockfileLocation: options.lockfileLocation,
			proxy: options.proxy,
			upgrade: options.upgrade,
			httpClient
		};
	},
	"compilation"
);
