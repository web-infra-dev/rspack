import { value } from "./provided-value";
import { process as collision } from "./collision";

it("should bind untransformed global references to the provided declaration", () => {
	expect(value).toBe("provided");
	expect(collision).toBe("collision");
});
