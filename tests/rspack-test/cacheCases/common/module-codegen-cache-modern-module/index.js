import { bridge } from "./bridge";
import { stable } from "./stable";

export { bridge, stable };

it("should reuse and invalidate modern-module codegen metadata", async () => {
	const asyncMod = await import("./async-module");

	expect(bridge).toBe(1);
	expect(stable).toBe("stable");
	expect(asyncMod.value).toBe(42);

	await NEXT_START();
});
---
import { bridge } from "./bridge";
import { stable } from "./stable";

export { bridge, stable };

it("should reuse and invalidate modern-module codegen metadata", async () => {
	const asyncMod = await import("./async-module");

	expect(bridge).toBe(1);
	expect(stable).toBe("stable");
	expect(asyncMod.value).toBe(42);

	await NEXT_START();
});
---
import { bridge } from "./bridge";
import { stable } from "./stable";

export { bridge, stable };

it("should reuse and invalidate modern-module codegen metadata", async () => {
	const asyncMod = await import("./async-module");

	expect(bridge).toBe(1);
	expect(stable).toBe("stable");
	expect(asyncMod.value).toBe(42);
});
