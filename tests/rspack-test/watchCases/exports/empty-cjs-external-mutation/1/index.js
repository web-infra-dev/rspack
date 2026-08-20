import "./mutator";
import { value } from "./barrel";

const findModule = name => __STATS__.modules.find(m => m.name.endsWith(`/${name}`));

it("should invalidate empty exports when an external mutation is added", () => {
	expect(value).toBe("ok");
	expect(findModule("empty.js").providedExports).toBe(null);
});
