import * as styles from "./style.module.css";

import.meta.webpackHot.accept(["./style.module.css"])

it("should work", async () => {
	expect(styles).toMatchObject({ class: "_style_module_css-class" });
	let firstFullHash = __webpack_hash__;
	await NEXT_HMR();
	expect(__webpack_hash__).not.toBe(firstFullHash);
});
