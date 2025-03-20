import { BuiltinPluginName, registerHttpClient } from "@rspack/binding";

import { create } from "./base";

// Interface for HttpUriPlugin options
interface HttpUriPluginOptions {
	/**
	 * HTTP client implementation for making requests
	 */
	httpClient?: {
		get: (url: string, headers: Record<string, string>) => Promise<any>;
	};
	cacheLocation?: string;
	frozen?: boolean;
	lockfileLocation?: string;
	proxy?: string;
	upgrade?: boolean;
	[key: string]: any;
}

// Create a default fetch implementation
const createDefaultFetch = () => {
	return (url: string, headers: Record<string, string>) => {
		return fetch(url, { headers }).then(async res => {
			// Get the response body as text
			const bodyText = await res.text();

			// Convert headers to a simple object
			const responseHeaders: Record<string, string> = {};
			res.headers.forEach((value, key) => {
				responseHeaders[key] = value;
			});

			// Return a structured response
			return {
				status: res.status,
				headers: responseHeaders,
				body: bodyText
			};
		});
	};
};

// Track if we've registered the HTTP client
let isHttpClientRegistered = false;

// Register the HTTP client with the Rust side
const registerHttpClientIfNeeded = (
	httpClient: (url: string, headers: Record<string, string>) => Promise<any>
) => {
	if (!isHttpClientRegistered) {
		try {
			// Register the real HTTP client function that will be called from Rust
			registerHttpClient(
				async (url: string, headers: Record<string, string>) => {
					try {
						// Call the provided HTTP client implementation
						const response = await httpClient(url, headers);
						return response;
					} catch (error: unknown) {
						// Handle any errors that occurred during the HTTP request
						console.error(`Error fetching ${url}:`, error);

						// Extract error message safely from the unknown error
						let errorMessage = "Unknown error";
						if (error instanceof Error) {
							errorMessage = error.message;
						} else if (typeof error === "string") {
							errorMessage = error;
						} else if (
							error &&
							typeof error === "object" &&
							"message" in error
						) {
							errorMessage = String(error.message);
						}

						return {
							status: 500,
							headers: { "content-type": "text/plain" },
							body: `Error fetching ${url}: ${errorMessage}`
						};
					}
				}
			);
			isHttpClientRegistered = true;
		} catch (e) {
			console.warn("Failed to register HTTP client with Rust:", e);
		}
	}
};

// Create the HttpUriPlugin
export const HttpUriPlugin = create(
	BuiltinPluginName.HttpUriPlugin,
	(options: boolean | HttpUriPluginOptions | undefined) => {
		// Convert boolean to empty object
		const normalizedOptions: HttpUriPluginOptions =
			options === true ? {} : options || {};

		// Use provided httpClient or create a default one
		const httpClient =
			normalizedOptions.httpClient?.get || createDefaultFetch();

		// Register our HTTP client with Rust
		registerHttpClientIfNeeded(httpClient);

		return normalizedOptions;
	},
	"compilation"
);
