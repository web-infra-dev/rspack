import fieldDisabledContext from "./context-field-disabled";
import fieldEnabledContext from "./context-field-enabled";

it("should control import.meta.webpackContext with importMeta.webpackContext", () => {
	expect(fieldEnabledContext).toBe("context-value");
	expect(fieldDisabledContext).toBe("undefined");
});
