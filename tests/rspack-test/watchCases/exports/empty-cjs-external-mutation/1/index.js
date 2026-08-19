import "./mutator";
import { value } from "./barrel";

it("should invalidate empty exports when an external mutation is added", () => {
	expect(value).toBe("ok");
});
