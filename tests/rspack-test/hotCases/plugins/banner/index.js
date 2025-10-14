import { value } from './file';

it("should inject banner to hot-update.js to update bannerIndex", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	expect(value).toBe(global.bannerIndex);
	NEXT(require("@rspack/test-tools/helper/legacy/update")(done, true, () => {
		expect(value).toBe(global.bannerIndex);
		NEXT(require("@rspack/test-tools/helper/legacy/update")(done, true, () => {
			expect(value).toBe(global.bannerIndex);
			NEXT(require("@rspack/test-tools/helper/legacy/update")(done, true, () => {
				expect(value).toBe(global.bannerIndex);
				delete global.bannerIndex;
				done();
			}))
		}));
	}));
}));

module.hot.accept("./file");
