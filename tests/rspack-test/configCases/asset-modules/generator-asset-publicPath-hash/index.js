import url from "../_images/file.png";

it("should import asset with module.generator.asset.publicPath", () => {
	expect(url).toMatch(/^[a-f0-9]{16}\/assets\/[a-f0-9]{10}\.file\.png$/);
	const assetInfo = __STATS__.assets.find(
		a => a.info.sourceFilename === "../_images/file.png"
	).info;
	expect(assetInfo.immutable).toBe(true);
	/** @NOTICE rspack's hash list is unordered */
	const contentHashLenList = assetInfo.contenthash.map(v => v.length).sort();
	expect(contentHashLenList.length).toBe(2);
	expect(contentHashLenList[0]).toBe(10);
	expect(contentHashLenList[1]).toBe(16);
});
