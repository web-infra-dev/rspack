import uiLib from 'ui-lib';

const fs = __non_webpack_require__("fs");
const path = __non_webpack_require__("path");

it("should generate share container bundle with expected content", async () => {
	const bundlePath = path.join(
		__dirname,
		"independent-share-host",
		"ui_lib/1.0.0",
		"share-entry.js"
	);
	const shareContainer = __non_webpack_require__(bundlePath).ui_lib;
	expect(Object.getOwnPropertyNames(shareContainer).sort()).toEqual(['get','init']);
	// TODO add init logic
	// await shareContainer.init();
	const shareModules = await shareContainer.get();
	expect(['Badge', 'Button', 'List',].every(m=>Boolean(shareModules[m])));
});
