import { foo as foo1 } from './a/foo.mjs'
import { foo as foo2 } from './_a/foo.mjs'

it("foo1 and foo2 should from different namespace import identifier", () => {
	expect(foo1).toBe("foo1");
	expect(foo2).toBe("foo2");
});
