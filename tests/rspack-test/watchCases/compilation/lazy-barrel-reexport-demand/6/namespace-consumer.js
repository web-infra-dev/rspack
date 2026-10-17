import * as star from "./lib";
import * as named from "./lib/named";

it("should keep namespace getters without changing chunk graph topology", () => {
	expect(star.create("star").value).toBe("star");
	expect(named.create("named").value).toBe("named");
});
