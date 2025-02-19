import { foo } from "./shared";

it("should compile fine", () => {
	expect(foo).toBe("foo");
	STATE.hash = STATS_JSON.assetsByChunkName.async[0];
	console.log(STATE)
});

it("should load the async chunk", () => {
	return import(/* webpackChunkName: "async" */ "./other-chunk");
});
