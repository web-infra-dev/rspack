import type { Compiler } from ".";

export class BrowserHttpImportPlugin {
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

	buildEsmUrl(moduleName: string) {
		return `https://esm.sh/${moduleName}`;
	}
}

function isUrl(request: string) {
	try {
		new URL(request);
		return true;
	} catch {
		return false;
	}
}
