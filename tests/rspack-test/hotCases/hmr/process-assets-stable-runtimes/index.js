import { value } from "./leaf";

it("should emit a changed shared module for every stable runtime", async () => {
	expect(value).toBe("before");
	await NEXT_HMR();
	expect(value).toBe("after");
});

import.meta.webpackHot.accept("./leaf");
