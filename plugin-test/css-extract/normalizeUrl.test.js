const normalizeUrl = require("../../packages/rspack/dist/builtin-plugin/css-extract/hmr/normalize-url");
const dataUrls = require("./fixtures/json/data-urls.json");

describe("normalize-url", () => {
	dataUrls.main.forEach(entry => {
		const [url, expected] = entry;

		it(`should work with "${url}" url`, async () => {
			const result = normalizeUrl(url);

			expect(result).toBe(expected);
		});
	});
});
