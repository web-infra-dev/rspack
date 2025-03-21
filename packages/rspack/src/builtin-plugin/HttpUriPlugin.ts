import { BuiltinPluginName, registerHttpClient } from "@rspack/binding";
import { create } from "./base";

export type HttpClientFunction = (
	url: string,
	headers: Record<string, string>
) => Promise<{
	status: number;
	headers: Record<string, string>;
	body: Buffer;
}>;

export type HttpUriPluginOptions = {
	/**
	 * The allowed URIs regexp
	 */
	allowedUris?: string[];
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
	/**
	 * HTTP client to use for requests
	 * Direct function that handles HTTP requests
	 */
	http_client?: HttpClientFunction;
};

/**
 * Plugin that allows loading modules from HTTP URLs
 */
export const HttpUriPlugin = create(
	BuiltinPluginName.HttpUriPlugin,
	(options: HttpUriPluginOptions = {}) => {
		// Register the HTTP client with adaptive parameter handling
		registerHttpClient((url, headers) => {
			// The parameters might be switched or in a different order
			// Extract the URL based on what we actually receive
			let actualUrl: string;
			let actualHeaders: Record<string, string> = {};

			if (typeof url === "string") {
				actualUrl = url;
				if (headers && typeof headers === "object") {
					actualHeaders = headers;
				}
			} else if (
				url &&
				typeof url === "object" &&
				headers &&
				typeof headers === "string"
			) {
				// Parameters are flipped
				actualUrl = headers;
				actualHeaders = url as unknown as Record<string, string>;
			} else if (
				url &&
				typeof url === "object" &&
				"url" in url &&
				typeof url.url === "string"
			) {
				// URL is inside an object
				actualUrl = url.url;
				if ("headers" in url && typeof url.headers === "object") {
					actualHeaders = url.headers as Record<string, string>;
				}
			} else {
				throw new Error(
					`Invalid parameters: url=${JSON.stringify(url)}, headers=${JSON.stringify(headers)}`
				);
			}

			console.log("\n🔥 Fetching URL:", actualUrl);

			// Return a promise that resolves to the response
			return fetch(actualUrl, { headers: actualHeaders }).then(response => {
				// Convert the response to the format expected by the HTTP client
				return response.arrayBuffer().then(buffer => {
					// Extract headers
					const responseHeaders: Record<string, string> = {};
					response.headers.forEach((value, key) => {
						responseHeaders[key] = value;
					});

					console.log(
						`✅ Fetched ${actualUrl} successfully (${buffer.byteLength} bytes)`
					);

					// Return the standardized format
					return {
						status: response.status,
						headers: responseHeaders,
						body: Buffer.from(buffer)
					};
				});
			});
		});

		const { http_client, ...restOptions } = options;

		return restOptions;
	},
	"thisCompilation"
);
