import "lib-js/a";
import.meta.webpackHot.accept();

it("should work if there are new initial chunks", async () => {
	expect((await import("./initial")).value).toBe("a");
	await NEXT_HMR();
});
---
import "lib-js/a";

it("should work if there are new initial chunks", async () => {
	await import("./initial");
	const installedChunks = __webpack_require__.hmrS_jsonp;
	expect(
		(installedChunks && installedChunks.lib === 0) ||
		globalThis.__INITIAL_CHUNK_HMR_CHUNKS__.some(([chunkIds]) =>
			chunkIds.includes("lib")
		)
	).toBe(true);
});
