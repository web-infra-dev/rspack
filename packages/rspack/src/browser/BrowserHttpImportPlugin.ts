import type { Compiler } from ".";

const DOMAIN_ESM_SH = "https://esm.sh";

interface BrowserHttpImportPluginOptions {
	domain?: string | ((request: string, packageName: string) => string);
}

export class BrowserHttpImportPlugin {
	constructor(private options: BrowserHttpImportPluginOptions = {}) {}

	apply(compiler: Compiler) {
		compiler.hooks.normalModuleFactory.tap("BrowserHttpImportPlugin", nmf => {
			nmf.hooks.resolve.tap("BrowserHttpImportPlugin", resolveData => {
				const request = resolveData.request;

				const issuerUrl = toUrl(resolveData.contextInfo.issuer);
				if (issuerUrl) {
					// If the issuer is a URL, request must be relative to that URL too
					resolveData.request = this.resolveWithUrlIssuer(issuerUrl, request);
					return;
				}

				if (this.isNodeModule(request)) {
					resolveData.request = this.resolveNodeModule(request);
					return;
				}
			});
		});
	}

	resolveWithUrlIssuer(issuer: URL, request: string) {
		return new URL(request, issuer).href;
	}

	resolveNodeModule(request: string) {
		return this.buildEsmUrl(request);
	}

	isNodeModule(request: string) {
		// Skip requess like "http://xxx"
		if (toUrl(request)) {
			return false;
		}

		// Skip relative requests
		return (
			!request.startsWith(".") &&
			!request.startsWith("/") &&
			!request.startsWith("!")
		);
	}

	buildEsmUrl(request: string) {
		let domain = DOMAIN_ESM_SH;
		if (typeof this.options.domain === "function") {
			const packageName = getPackageName(request);
			domain = this.options.domain(request, packageName);
		} else if (typeof this.options.domain === "string") {
			domain = this.options.domain;
		}
		return `${domain}/${request}`;
	}
}

function getPackageName(request: string) {
	if (request.startsWith("@")) {
		const parts = request.split("/");
		return `${parts[0]}/${parts[1]}`;
	}
	return request.split("/")[0];
}

function toUrl(request: string): URL | undefined {
	try {
		const url = new URL(request);
		return url;
	} catch {
		return undefined;
	}
}
