import * as styles from "./style.module.css";
import update from "../../update.esm";

import.meta.webpackHot.accept(["./style.module.css"])

it("should work", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	expect(styles).toMatchObject({ class: "_style_module_css-class" });
	let firstFullHash = __webpack_hash__;

	NEXT(update(done, true, () => {
		try{
			// only css change should also trigger changing full hash in runtime
			expect(__webpack_hash__).not.toBe(firstFullHash);
			done();
		}catch(e){
			done(e)
		}
	}));
}));
