import "./webpack-20320";
import "./webpack-20561";
import * as foo from "./foo";
import { obj } from "./foo";

const missing = "d" in foo ? foo.d() : "nope";
const unguarded = "d" in foo ? foo.e : "nope";
const present = "c" in foo ? foo.c() : "nope";
const hasC = "c" in foo;
const namedMissing = "d" in obj ? obj.d : "nope";
const namedUnguarded = "d" in obj ? obj.e : "nope";
const namedPresent = "x" in obj ? obj.x() : "nope";
const nestedMissing = "d" in foo.obj ? foo.obj.d : "nope";
const nestedUnguarded = "d" in foo.obj ? foo.obj.e : "nope";
const nestedPresent = "x" in foo.obj ? foo.obj.x() : "nope";

expect(missing).toBe("nope");
expect(unguarded).toBe("nope");
expect(present).toBe("c");
expect(hasC).toBe(true);
expect(namedMissing).toBe("nope");
expect(namedUnguarded).toBe("nope");
expect(namedPresent).toBe("x");
expect(nestedMissing).toBe("nope");
expect(nestedUnguarded).toBe("nope");
expect(nestedPresent).toBe("x");
