import type { Compiler } from ".";

interface BrowserHttpImportPluginOptions {
	/**
	 * ESM CDN domain
	 */
	domain: string | ((request: string, packageName: string) => string);
	/**
	 * Specify ESM CDN URL for dependencies.
	 */
	dependencyUrl?:
		| Record<string, string | undefined>
		| ((packageName: string) => string | undefined);
	/**
	 * Specify versions for dependencies.
	 * Default to "latest" if not specified.
	 */
	dependencyVersions?: Record<string, string | undefined>;
}

/**
 * Convert imports of dependencies in node modules to http imports from esm cdn.
 */
export class BrowserHttpImportEsmPlugin {
	constructor(private options: BrowserHttpImportPluginOptions) {}

	apply(compiler: Compiler) {
		compiler.hooks.normalModuleFactory.tap("BrowserHttpImportPlugin", nmf => {
			nmf.hooks.resolve.tap("BrowserHttpImportPlugin", resolveData => {
				const request = resolveData.request;
				const packageName = getPackageName(request);

				// We don't consider match resource and inline loaders
				// Because usually they are not used with dependent modules like `sass-loader?react`
				if (request.includes("!")) {
					return;
				}

				// If dependencyUrl is provided, use it to resolve the request
				if (this.options.dependencyUrl) {
					if (typeof this.options.dependencyUrl === "function") {
						const url = this.options.dependencyUrl(packageName);
						if (url) {
							resolveData.request = url;
							return;
						}
					} else if (typeof this.options.dependencyUrl === "object") {
						const url = this.options.dependencyUrl[packageName];
						if (url) {
							resolveData.request = url;
							return;
						}
					}
				}

				// If the issuer is a URL, request must be relative to that URL too
				const issuerUrl = toHttpUrl(resolveData.contextInfo.issuer);
				if (issuerUrl) {
					resolveData.request = this.resolveWithUrlIssuer(request, issuerUrl);
					return;
				}

				// If the request is a node module, resolve it with esm cdn URL
				if (this.isNodeModule(request)) {
					resolveData.request = this.resolveNodeModule(request, packageName);
					return;
				}
			});
		});
	}

	resolveWithUrlIssuer(request: string, issuer: URL) {
		return new URL(request, issuer).href;
	}

	resolveNodeModule(request: string, packageName: string) {
		let domain = "";
		if (typeof this.options.domain === "function") {
			domain = this.options.domain(request, packageName);
		} else if (typeof this.options.domain === "string") {
			domain = this.options.domain;
		}

		const version = this.options.dependencyVersions?.[packageName] || "latest";
		const versionedRequest = getRequestWithVersion(request, version);
		return `${domain}/${versionedRequest}`;
	}

	isNodeModule(request: string) {
		// Skip requests like "http://xxx"
		if (toHttpUrl(request)) {
			return false;
		}

		// Skip relative requests
		return (
			!request.startsWith(".") &&
			!request.startsWith("/") &&
			!request.startsWith("!")
		);
	}
}

function getPackageName(request: string) {
	if (request.startsWith("@")) {
		const parts = request.split("/");
		return `${parts[0]}/${parts[1]}`;
	}
	return request.split("/")[0];
}

function getRequestWithVersion(request: string, version: string) {
	// Handle scoped packages (packages starting with '@')
	if (request.startsWith("@")) {
		// Find the position of the second '/' (if exists)
		const secondSlashIndex = request.indexOf("/", request.indexOf("/") + 1);

		if (secondSlashIndex === -1) {
			// No second '/', add version at the end
			return `${request}@${version}`;
		} else {
			// Has second '/', add version after the scoped package name
			const scopedPackage = request.substring(0, secondSlashIndex);
			const restPath = request.substring(secondSlashIndex);
			return `${scopedPackage}@${version}${restPath}`;
		}
	} else {
		// Non-scoped packages
		const firstSlashIndex = request.indexOf("/");

		if (firstSlashIndex === -1) {
			// No '/', add version at the end
			return `${request}@${version}`;
		} else {
			// Has '/', add version after the first package name
			const packageName = request.substring(0, firstSlashIndex);
			const restPath = request.substring(firstSlashIndex);
			return `${packageName}@${version}${restPath}`;
		}
	}
}

function toHttpUrl(request: string): URL | undefined {
	try {
		const url = new URL(request);
		if (url.protocol === "http:" || url.protocol === "https:") {
			return url;
		}
	} catch {
		return undefined;
	}
}
