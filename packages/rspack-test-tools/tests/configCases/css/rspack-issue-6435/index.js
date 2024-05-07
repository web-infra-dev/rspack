import * as classes from "./style.module.css";
import legacyClasses from "./legacy/index.css";

const crypto = require("crypto");
const createHash = str => crypto.createHash('md4').update(str).digest('hex').slice(0, 20).replace(/^\d+/, "");

it("should have consistent hash", () => {
	expect(classes["container-main"]).toBe(`${createHash("./style.module.css")}-container-main`)
	expect(legacyClasses["legacy-main"]).toBe(`${createHash("./index.css")}-legacy-main`)
});
