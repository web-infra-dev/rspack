import { value } from "./leaf";

it("should fall back when incremental chunk assets are disabled", async () => {
	expect(value).toBe("before");
	await NEXT_HMR();
	expect(value).toBe("after");
});

import.meta.webpackHot.accept("./leaf");
