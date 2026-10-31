const readGlobals = require("./globals");
var value = "outer";
let caught;
try {
	throw "caught";
} catch (value) {
	var value = "changed";
	caught = () => ({ value });
}

if (true) {
	function legacy() {
		return legacy;
	}
}

it("renames catch vars and Annex B functions when inlining a script entry", () => {
	expect(value).toBe("outer");
	expect(caught()).toEqual({ value: "changed" });
	expect(legacy()).toBe(legacy);
	expect(readGlobals()).toEqual(["undefined", "undefined"]);
});
