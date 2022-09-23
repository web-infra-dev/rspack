import getCurrentScriptSource from "./getCurrentScriptSource.js";

function parseURL(resourceQuery: string): Record<string, string | boolean> {
	let options: Record<string, string> = {};

	if (typeof resourceQuery === "string" && resourceQuery !== "") {
		const searchParams = resourceQuery.slice(1).split("&");

		for (let i = 0; i < searchParams.length; i++) {
			const pair = searchParams[i].split("=");

			options[pair[0]] = decodeURIComponent(pair[1]);
		}
	} else {
		// Else, get the url from the <script> this file was called with.
		const scriptSource = getCurrentScriptSource();

		let scriptSourceURL: URL;

		try {
			// The placeholder `baseURL` with `window.location.href`,
			// is to allow parsing of path-relative or protocol-relative URLs,
			// and will have no effect if `scriptSource` is a fully valid URL.
			scriptSourceURL = new URL(scriptSource, self.location.href);
		} catch (error) {
			// URL parsing failed, do nothing.
			// We will still proceed to see if we can recover using `resourceQuery`
		}

		if (scriptSourceURL) {
			// @ts-ignored
			options = scriptSourceURL;
			// @ts-ignored
			options.fromCurrentScript = true;
		}
	}

	return options;
}

export default parseURL;
