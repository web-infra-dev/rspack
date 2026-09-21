import { checkUrls } from "./check-urls.js";

it("should promote JavaScript URLs inside nested async blocks without executing them", async () => {
	const [jsUrlA, jsUrlB, assetUrl, jsAssetUrl] = await new Promise((resolve, reject) => {
		require.ensure([], require => {
			const jsUrlA = new URL("./target-a.js", import.meta.url);
			require.ensure([], require => {
				resolve([
					jsUrlA,
					new URL("./target-b.js", import.meta.url),
					new URL("./target.png", import.meta.url),
					new URL("./target-asset.js", import.meta.url),
				]);
			}, reject, "inner");
		}, reject, "outer");
	});
	checkUrls([jsUrlA, jsUrlB, assetUrl, jsAssetUrl]);
});
