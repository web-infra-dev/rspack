import value from "./reexport-async-unknown.js?default";
import { a, b, c } from "./reexport-async-unknown.js?named";
import * as ns from "./reexport-async-unknown.js?ns";

function nsObj(m) {
	Object.defineProperty(m, Symbol.toStringTag, { value: "Module" });
	return m;
}

expect(ns).toEqual(
	nsObj({
		default: "default",
		a: "a",
		b: "b",
		c: "c"
	})
);

expect(a).toBe("a");
expect(b).toBe("b");
expect(c).toBe("c");

expect(value).toBe("default");
