import { val } from "./module";

it("should accept changes", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	expect(val).toBe(1);
	NEXT(require("@rspack/test-tools/helper/legacy/update")(done, true, () => done()));
}));
