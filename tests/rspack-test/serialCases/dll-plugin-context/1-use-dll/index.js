import { hello, loadWidget } from "dll/index";

it("should consume a DLL that includes a context module", async function () {
	expect(hello).toBe("hello");
	const mod = await loadWidget("a");
	expect(mod.default).toBe("a");
});

it("should write redirect-warn (not redirectWarn) for context modules", function () {
	const manifest = require("../../../js/config/dll-plugin-context/manifest0.json");
	const contextKey = Object.keys(manifest.content).find(k =>
		k.includes("lazy recursive")
	);
	expect(contextKey).toBeTruthy();
	expect(manifest.content[contextKey].buildMeta.defaultObject).toBe(
		"redirect-warn"
	);
});
