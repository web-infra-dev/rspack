import { a, usedExports } from "./reexport?1";
import reexport2 from "./reexport?2";
import * as reexport3 from "./reexport?3";

it("tree shaking should works", () => {
    expect(a()).toBe(1);
    expect(usedExports).toEqual(["a", "usedExports"]);
    expect(reexport2.a()).toBe(1);
    expect(reexport2.usedExports).toEqual(["a", "usedExports"]);
    expect(reexport3.a()).toBe(1);
    expect(reexport3.usedExports).toEqual(["a", "usedExports"]);
})
