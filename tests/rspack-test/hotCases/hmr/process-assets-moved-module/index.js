import { anchor } from "./shared/anchor";
import { moving } from "./shared/moving";

it("should update an installed chunk when an edited module moves", async () => {
	expect(anchor + moving).toBe("anchormoving-before");
	await NEXT_HMR();
	expect(anchor + moving).toBe("anchormoving-after");
});

import.meta.webpackHot.accept("./shared/moving");
