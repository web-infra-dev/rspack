import { checkUrls } from "./check-urls.js";
import "./nested.js";

const jsUrlA = new URL("./target-a.js", import.meta.url);
const jsUrlB = new URL("./target-b.js", import.meta.url);
const assetUrl = new URL("./target.png", import.meta.url);
const jsAssetUrl = new URL("./target-asset.js", import.meta.url);

it("should turn JavaScript URL dependencies into async entries", () => {
	checkUrls([jsUrlA, jsUrlB, assetUrl, jsAssetUrl]);
});
