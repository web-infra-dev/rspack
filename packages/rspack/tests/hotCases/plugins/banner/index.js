import { value } from './file';

it("should inject banner to hot-update.js to update bannerIndex", (done) => {
	expect(value).toBe(global.bannerIndex);
	NEXT(require("../../update")(done, true, () => {
		expect(value).toBe(global.bannerIndex);
		NEXT(require("../../update")(done, true, () => {
			expect(value).toBe(global.bannerIndex);
			NEXT(require("../../update")(done, true, () => {
				expect(value).toBe(global.bannerIndex);
				done();
			}))
		}));
	}));
});

module.hot.accept("./file");
