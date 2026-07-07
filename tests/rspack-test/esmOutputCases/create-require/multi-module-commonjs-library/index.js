import { required } from "./only-require.js";
import { resolved } from "./with-resolve.js";

export { required, resolved };

it("strips created require declarations in commonjs library output", () => {
	expect(required).toBe("dep");
	expect(resolved).toBe("./dep.js");
});
