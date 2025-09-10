import type { Compiler } from ".";

interface ResolvedRequest {
	request: string;
	issuer: string;
	packageName: string;
}

interface ProcessedRequest extends ResolvedRequest {
	url: URL;
}

interface BrowserHttpImportPluginOptions {
	/**
	 * ESM CDN domain
	 */
	domain: string | ((resolvedRequest: ResolvedRequest) => string);
	/**
	 * Specify ESM CDN URL for dependencies.
	 * If a record is provided, it will be used to map package names to their CDN URLs.
	 *
	 * Once this function resolves a dependency, other options are ignored.
	 */
	dependencyUrl?:
		| Record<string, string | undefined>
		| ((resolvedRequest: ResolvedRequest) => string | undefined);
	/**
	 * Specify versions for dependencies.
	 * Default to "latest" if not specified.
	 */
	dependencyVersions?: Record<string, string | undefined>;
	/**
	 * You can attach additional queries supported by the CDN to the `request.url`.
	 *
	 * For example, to specify the external dependencies under esm.sh, you can do:
	 *
	 * `request.url.searchParams.set("external", "react,react-dom")`
	 */
	postprocess?: (request: ProcessedRequest) => void;
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
				const issuer = resolveData.contextInfo.issuer;

				// We don't consider match resource and inline loaders
				// Because usually they are not used with dependent modules like `sass-loader?react`
				if (request.includes("!")) {
					return;
				}

				// There are some common cases of request and issuer:
				// 1. Relative request + path issuer
				// 2. Relative request + http issuer
				// 3. Absolute request + http issuer
				// 4. Node module specifier + path issuer
				const issuerUrl = toHttpUrl(issuer);
				const resolvedRequest = resolveRequest(request, issuer, !!issuerUrl);

				// If dependencyUrl is provided, use it to resolve the request
				if (this.options.dependencyUrl) {
					if (typeof this.options.dependencyUrl === "function") {
						const url = this.options.dependencyUrl(resolvedRequest);
						if (url) {
							resolveData.request = url;
							return;
						}
					} else if (typeof this.options.dependencyUrl === "object") {
						const url = this.options.dependencyUrl[resolvedRequest.packageName];
						if (url) {
							resolveData.request = url;
							return;
						}
					}
				}

				// If the issuer is a URL, request should base on that
				if (issuerUrl) {
					resolveData.request = this.parameterize(
						this.resolveWithUrlIssuer(request, issuerUrl),
						resolvedRequest
					);
					return;
				}

				// If the request is a node module, resolve it with esm cdn URL
				if (this.isNodeModule(request)) {
					resolveData.request = this.parameterize(
						this.resolveNodeModule(resolvedRequest),
						resolvedRequest
					);
					return;
				}
			});
		});
	}

	private resolveWithUrlIssuer(request: string, issuer: URL) {
		return new URL(request, issuer).href;
	}

	private resolveNodeModule(resolvedRequest: ResolvedRequest) {
		let domain = "";
		if (typeof this.options.domain === "function") {
			domain = this.options.domain(resolvedRequest);
		} else if (typeof this.options.domain === "string") {
			domain = this.options.domain;
		}

		const version =
			this.options.dependencyVersions?.[resolvedRequest.packageName] ||
			"latest";
		const versionedRequest = getRequestWithVersion(
			resolvedRequest.request,
			version
		);
		return `${domain}/${versionedRequest}`;
	}

	private parameterize(requestUrl: string, resolvedRequest: ResolvedRequest) {
		if (!this.options.postprocess) {
			return requestUrl;
		}

		const url = new URL(requestUrl);
		this.options.postprocess({
			url,
			...resolvedRequest
		});
		return url.toString();
	}

	private isNodeModule(request: string) {
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

function resolveRequest(
	request: string,
	issuer: string,
	isHttpIssuer: boolean
): ResolvedRequest {
	function resolvePackageName() {
		// Such as "/react@19.1.1/es2022/react.mjs" and "/@module-federation/runtime@0.18.3/es2022/runtime.mjs"
		if (isHttpIssuer) {
			// If the issuer is a URL, the request is usually an absolute URL with explicit version
			// But if the request is a relative path, we should resolve based on the issuer
			let requestToResolve = request;
			if (!request.startsWith("/")) {
				requestToResolve = issuer;
			}

			// ['', '@module-federation', 'runtime@0.18.3', 'es2022', 'runtime.mjs']
			const segments = requestToResolve.split("/");

			// Find the first segment with "@" but not starting with "@"
			const nameSegIndex = segments.findIndex(
				segment => segment.includes("@") && !segment.startsWith("@")
			);

			if (nameSegIndex > 0) {
				const nameSeg = segments[nameSegIndex];
				const atIndex = nameSeg.indexOf("@");
				const name = nameSeg.slice(0, atIndex);

				// If group segment which starts with "@" exists
				const groupSeg = segments[nameSegIndex - 1] ?? "";
				if (groupSeg.startsWith("@")) {
					return `${groupSeg}/${name}`;
				} else {
					return name;
				}
			}
		}

		if (request.startsWith("@")) {
			const parts = request.split("/");
			return `${parts[0]}/${parts[1]}`;
		}
		return request.split("/")[0];
	}

	const packageName = resolvePackageName();

	return {
		packageName,
		request,
		issuer
	};
}

/**
 * This function is called only when the request is a node module specifier,
 * i.e. isNodeModule(request) === true
 */
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
