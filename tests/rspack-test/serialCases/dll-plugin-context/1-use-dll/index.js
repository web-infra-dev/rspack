import { hello, loadWidget } from "dll/index";
import {
	hello as camelHello,
	loadWidget as camelLoadWidget
} from "camel-dll/index";

it("should consume a DLL manifest that includes a context module", async function () {
	expect(hello).toBe("hello");
	const mod = await loadWidget("a");
	expect(mod.default).toBe("a");
});

it("should consume a camelCase DLL manifest that includes a context module", async function () {
	expect(camelHello).toBe("hello");
	const mod = await camelLoadWidget("a");
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
