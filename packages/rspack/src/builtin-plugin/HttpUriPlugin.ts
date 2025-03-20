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

/**
 * Default HTTP client implementation using the fetch API
 */
const defaultHttpClient: HttpClientFunction = (url, headers) => {
	return fetch(url, { headers })
		.then(response => {
			return response.arrayBuffer().then(buffer => {
				// Convert headers to record
				const responseHeaders: Record<string, string> = {};
				response.headers.forEach((value, key) => {
					responseHeaders[key] = value;
				});

				return {
					status: response.status,
					headers: responseHeaders,
					body: Buffer.from(buffer)
				};
			});
		})
		.catch(error => {
			throw error;
		});
};

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
		// Use provided HTTP client or fall back to default
		const httpClient = options.http_client || defaultHttpClient;

		// Always register an HTTP client
		registerHttpClient((err, method, url, headers) => {
			if (err) throw err;

			const safeUrl = typeof url === "string" && url ? url : String(url || "");

			const safeHeaders: Record<string, string> =
				headers && typeof headers === "object" ? headers : {};

			return httpClient(safeUrl, safeHeaders);
		});

		const { http_client, ...restOptions } = options;

		return restOptions;
	},
	"thisCompilation"
);
