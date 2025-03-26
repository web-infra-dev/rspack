import path from "node:path";
import {
	type BuiltinPlugin,
	BuiltinPluginName,
	type RawHttpUriPluginOptions
} from "@rspack/binding";
import type { Compiler } from "../Compiler";
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

const defaultHttpClient = (url: string, headers: Record<string, string>) => {
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
