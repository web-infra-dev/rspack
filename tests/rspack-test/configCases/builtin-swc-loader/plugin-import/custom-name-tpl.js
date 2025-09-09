import { FooBar as camel } from "./src/custom-name-tpl-camel";
import { FooBar as kebab } from "./src/custom-name-tpl-kebab";
import { FooBar as snake } from "./src/custom-name-tpl-snake";
import { FooBar as upper } from "./src/custom-name-tpl-upper";
import { FooBar as lower } from "./src/custom-name-tpl-lower";

it("custom-name-tpl", () => {
	expect(camel).toBe("FooBar");
	expect(kebab).toBe("FooBar");
	expect(snake).toBe("FooBar");
	expect(upper).toBe("FooBar");
	expect(lower).toBe("FooBar");
});
