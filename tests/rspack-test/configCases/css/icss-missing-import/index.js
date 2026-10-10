import * as values from "./value.module.css";
import * as imports from "./import.module.css";
import * as compositions from "./composes.module.css";

it("should finish code generation for unresolved ICSS imports", () => {
	expect(values.color).toBe("");
	expect(imports.color).toBe("");
	expect(typeof values.local).toBe("string");
	expect(typeof imports.local).toBe("string");
	expect(typeof compositions.local).toBe("string");
});
