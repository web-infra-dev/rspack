import * as collision from "./collision.module.css";
import * as repeated from "./repeated.module.css";
import * as transitive from "./transitive.module.css";
import * as composed from "./composed.module.css";

it("combines locals that share a camel-case export alias", () => {
	expect(collision.fooBar).toBe("foo-bar fooBar");
});

it("does not append a repeated local already present in the shared alias", () => {
	expect(repeated.fooBar).toBe("foo-bar fooBar");
});

it("does not append locals already present through multiple alias definitions", () => {
	expect(transitive.fooBar).toBe("foo-bar fooBar foo_bar");
});

it("preserves repeated composed classes from distinct alias members", () => {
	expect(composed.fooBar).toBe("foo-bar base fooBar base");
});
