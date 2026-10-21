import * as collision from "./collision.module.css";
import * as repeated from "./repeated.module.css";
import * as transitive from "./transitive.module.css";
import * as composed from "./composed.module.css";
import * as forward from "./forward.module.css";
import * as backward from "./backward.module.css";
import * as owner from "./owner.module.css";

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

it("resolves forward compositions using the exact local name", () => {
	expect(forward.target).toBe("target fooBar");
	expect(forward.dashedTarget).toBe("dashedTarget foo-bar");
	expect(forward.fooBar).toBe("foo-bar fooBar");
});

it("resolves existing compositions using the exact local name", () => {
	expect(backward.target).toBe("target fooBar");
	expect(backward.dashedTarget).toBe("dashedTarget foo-bar");
	expect(backward.fooBar).toBe("foo-bar fooBar");
});

it("attaches compositions to the exact local definition behind a shared alias", () => {
	expect(owner.target).toBe("target fooBar base");
	expect(owner.valueTarget).toBe("valueTarget fooBar base");
	expect(owner.fooBar).toBe("foo-bar fooBar base");
});
