export { value } from "./lib";

import { value } from "./lib";

it("keeps runtime module lexical variables scoped in ESM output", () => {
	expect(value).toBe(42);
});
