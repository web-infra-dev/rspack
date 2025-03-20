// Plugin class for handling HTTP URI modules
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
		// Extract http_client if provided and register it with the native side
		if (options.http_client) {
			// The registerHttpClient function expects a callback that properly matches
			// the ThreadsafeFunction's expected signature with error as first parameter
			registerHttpClient((err, _method, _url, url, headers) => {
				if (err) throw err;
				return options.http_client!(url, headers);
			});
		}

		// Do not pass http_client to Rust since it's registered separately
		const { http_client, ...restOptions } = options;

		// Only return the other options
		return restOptions;
	},
	"thisCompilation"
);
