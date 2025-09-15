import "./module";
import "./cjs";
import "./pure";
import "./referenced";
import "./side-referenced";

if (process.env.NODE_ENV === "production") {
	it("should skip imports to empty modules", () => {
		expect(require.resolveWeak("./cjs")).toBe(null);
		expect(require.resolveWeak("./module")).toBe(null);
		expect(require.resolveWeak("./pure")).toBe(null);
		expect(require.resolveWeak("./referenced")).toBe(null);
	});
}

it("should not skip transitive side effects", () => {
	expect(global.__test_value__).toBe(true);
	delete global.__test_value__;
});
