import { fsNs } from "./namespace";
import { readFile, readFileSync } from "./named";

it("should keep node-commonjs namespace and named external imports aligned", () => {
	expect(fsNs.readFile).toBe(readFile);
	expect(fsNs.readFileSync).toBe(readFileSync);
});
