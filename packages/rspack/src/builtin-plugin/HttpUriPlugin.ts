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
	allowedUris?: RegExp[];
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
		// No need to normalize - only use snake_case http_client
		// Register the HTTP client directly
		if (options.http_client) {
			registerHttpClient(options.http_client);
		}

		return options;
	},
	"thisCompilation"
);
