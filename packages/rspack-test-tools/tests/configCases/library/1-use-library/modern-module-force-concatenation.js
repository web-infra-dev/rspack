import path from 'path'
import { a } from "library/a.js";
import { foo as eFoo, bar as eBar } from "library/e.js";
import { value as f } from "library/f.js";
import { foo as g } from "library/g.js";

it("should get default export from library (" + NAME + ")", function() {
	expect(a).toBe("a");
	// b: TODO: CJS module can't be exported in "module"
	// c: no exports
	// d: no exports
	expect(eFoo).toBe("foo");
	expect(eBar).toBe("bar");
	expect(f).toBe(`foo${path.sep}`);
	expect(g).toBe("foo");
});
