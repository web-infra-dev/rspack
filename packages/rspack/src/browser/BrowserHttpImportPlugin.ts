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
				if (!this.isNodeModule(request)) {
					return;
				}
				const esmUrl = this.buildEsmUrl(request);
				resolveData.request = esmUrl;
			});
		});
	}

	isNodeModule(request: string) {
		// Skip requess like "http://xxx"
		if (isUrl(request)) {
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

function isUrl(request: string) {
	try {
		new URL(request);
		return true;
	} catch {
		return false;
	}
}
