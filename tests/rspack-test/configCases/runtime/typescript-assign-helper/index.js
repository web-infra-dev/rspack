import { merge as mergeA } from "./merge-a";
import { merge as mergeB } from "./merge-b";

it("shares the TypeScript assign fallback without changing its behavior", () => {
	const originalAssign = Object.assign;
	Object.assign = undefined;
	try {
		const inherited = { inherited: true };
		const source = Object.create(inherited);
		source.own = "value";

		expect(mergeA({ a: 1 }, source)).toEqual({ a: 1, own: "value" });
		expect(mergeB({ b: 2 }, { c: 3 })).toEqual({ b: 2, c: 3 });
	} finally {
		Object.assign = originalAssign;
	}
});
