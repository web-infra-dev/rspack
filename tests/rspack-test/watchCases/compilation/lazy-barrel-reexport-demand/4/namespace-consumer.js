import * as star from "./lib";
import * as named from "./lib/named";

it("should access only local exports through stable barrel connections", () => {
	expect(star.create("star").value).toBe("star");
	expect(named.create("named").value).toBe("named");
});
