import {val} from "./module";

it("should accept changes", (done) => {
	expect(val).toBe(1);
	// CHANGE: fix an issue when `done` is called before compiler is done, causing a segmentation fault.
	NEXT(require("../../update")(done, true, () => done()));
});
