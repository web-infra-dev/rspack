import uiLib from 'ui-lib';

const fs = __non_webpack_require__("fs");
const path = __non_webpack_require__("path");

it("should generate share container bundle with expected content", async () => {
	const bundlePath = path.join(
		__dirname,
		"1.0.0",
		"share-entry.js"
	);
	const shareContainer = __non_webpack_require__(bundlePath).ui_lib;
	expect(Object.getOwnPropertyNames(shareContainer).sort()).toEqual(['get','init']);
	__webpack_require__.consumesLoadingData = {
		initialConsumes: {
		}
	}
	const response = await shareContainer.init({},{
		installInitialConsumes: async ()=>{
			return 'call init'
		}
	});
	expect(response).toBe('call init');
	const shareModules = await shareContainer.get();
	expect(['Button', 'Badge', 'List', 'MessagePro', 'SpinPro'].every(m=>Boolean(shareModules[m])));
});
