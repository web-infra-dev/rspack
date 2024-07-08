import {
	noop,
	reassign,
	rest,
	item1,
	items,
	rename,
	notExist // not exist
} from "./a.js";

noop(reassign, rest, item1, items);
noop(rename, notExist); // must use import for error reporting

it("should not warn for export destructring", () => {
	expect(1).toBe(1);
});
