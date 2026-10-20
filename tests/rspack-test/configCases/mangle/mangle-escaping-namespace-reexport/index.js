// Adapted from webpack's optimization/mangled-namespace-default-reexport (MIT).
import fromSpecifier from "./specifier";
import fromExpression from "./expression";

it("resolves static members through namespaces re-exported as default", () => {
	expect(fromSpecifier.member.value).toBe(42);
	expect(fromExpression.member.value).toBe(42);
});
