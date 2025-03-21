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

const defaultHttpClient: HttpClientFunction = (url, headers) => {
	console.log("\n🔥 Fetching URL:", url);

	return fetch(url, { headers }).then(response => {
		return response.arrayBuffer().then(buffer => {
			const responseHeaders: Record<string, string> = {};
			response.headers.forEach((value, key) => {
				responseHeaders[key] = value;
			});

			console.log(
				`✅ Fetched ${url} successfully (${buffer.byteLength} bytes)`
			);

			return {
				status: response.status,
				headers: responseHeaders,
				body: Buffer.from(buffer)
			};
		});
	});
};

/**
 * Plugin that allows loading modules from HTTP URLs
 */
export const HttpUriPlugin = create(
	BuiltinPluginName.HttpUriPlugin,
	(options: HttpUriPluginOptions = {}) => {
		const { http_client, ...restOptions } = options;

		registerHttpClient(http_client || defaultHttpClient);

		return restOptions;
	},
	"thisCompilation"
);
