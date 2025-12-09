import { value } from './file';

it("should inject banner to hot-update.js to update bannerIndex", async () => {
	expect(value).toBe(globalThis.bannerIndex);
	await NEXT_HMR();
	expect(value).toBe(globalThis.bannerIndex);
	await NEXT_HMR();
	expect(value).toBe(globalThis.bannerIndex);
	await NEXT_HMR();
	expect(value).toBe(globalThis.bannerIndex);
	delete globalThis.bannerIndex;
});

module.hot.accept("./file");
