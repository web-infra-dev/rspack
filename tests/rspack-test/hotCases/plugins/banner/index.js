import { value } from './file';

it("should inject banner to hot-update.js to update bannerIndex", async () => {
	expect(value).toBe(global.bannerIndex);
	await NEXT_HMR();
	expect(value).toBe(global.bannerIndex);
	await NEXT_HMR();
	expect(value).toBe(global.bannerIndex);
	await NEXT_HMR();
	expect(value).toBe(global.bannerIndex);
	delete global.bannerIndex;
});

module.hot.accept("./file");
