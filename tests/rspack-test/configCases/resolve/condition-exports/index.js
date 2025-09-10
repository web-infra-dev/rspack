import esmImport from "exports-conditional";
import esmImport2 from "exports-conditional-2";

const cjsImport = require("exports-conditional");
const cjsImport2 = require("exports-conditional-2");

it("conditional exports should works", () => {
	expect(esmImport).toBe("esm");
	expect(cjsImport).toBe("cjs");

	expect(esmImport2).toBe("esm");
	expect(cjsImport2).toBe("cjs");
});
