import { loadAsset } from "./shared";
it("should retain URLs in the runtime using their export", async () => {
	const [asset, nested, script] = await loadAsset();
	expect([asset, nested]).toEqual(["/path/asset.txt", "/path/nested.txt"]);
	expect(script).toMatch(/^\/path\/\d+-0\.js$/);
	expect(globalThis.URL_RUNTIME_FILTER_TARGET_EXECUTED).toBeUndefined();
});
