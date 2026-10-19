import * as star from "./lib";
import * as named from "./lib/named";

it("should initially reference only local exports through the barrel connections", () => {
	expect(star.create("star").value).toBe("star");
	expect(named.create("named").value).toBe("named");
});
