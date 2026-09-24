import fs from "node:fs";
import path from "node:path";

const cssRequests = [];

export default {
	findBundle(index, options) {
		const name = options.name;
		const html = fs.readFileSync(
			path.join(options.output.path, `main.${name}.html`),
			"utf-8"
		);
		const stylesheets = Array.from(
			html.matchAll(/<link\b[^>]*href="([^"]+)"[^>]*>/g),
			([tag, href]) => {
				expect(tag).toContain('rel="stylesheet"');
				return href;
			}
		);
		const scripts = Array.from(
			html.matchAll(/<script\b[^>]*src="([^"]+)"[^>]*>/g),
			([, src]) => src
		);
		expect(stylesheets).toEqual(name.endsWith("-preloaded")
			? [`https://test.cases/path/shared-css.${name}.css`]
			: []);
		expect(scripts).toContain(`https://test.cases/path/main.${name}.js`);
		expect(scripts).not.toContain(`https://test.cases/path/other.${name}.js`);
		// Load the HTML plugin's stylesheets before its deferred scripts, as
		// a browser does. The other entry is never included in this page.
		return [...stylesheets, ...scripts].map(url =>
			path.posix.basename(new URL(url).pathname)
		);
	},
	resourceLoader(url) {
		if (url.endsWith(".css")) cssRequests.push(url);
	},
	moduleScope(scope, stats, options) {
		scope.runtimeChunkMode = options.name;
		scope.preloadedCss = options.name.endsWith("-preloaded");
		scope.getCssRequests = () => cssRequests.filter(url =>
			url.endsWith(`.${options.name}.css`)
		);
	}
};
