import * as styles1 from "./index.module.css"
import * as styles2 from "./deep/index.module.css"

it("should have different local ident name", async () => {
	expect(styles1.a).toBe("__index_module-a");
	expect(styles2.a).toBe("__deep_index_module-a");
})