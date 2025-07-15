import {
	aUsed,
	bUsed,
	cUsed
} from "root1";
import {
	dUsed,
	eUsed,
	fUsed
} from "root2";
import {
	_1, 
	e
} from "./path1";

it("should use only current entrypoint exports", () => {
	expect(e).toBe("e");
	expect(_1.a).toBe("a");
	expect(_1.c).toBe("c");
	expect(aUsed).toBe(true);
	expect(bUsed).toBe(false);
	expect(cUsed).toBe(true);
	expect(dUsed).toBe(false);
	expect(eUsed).toBe(true);
	expect(fUsed).toBe(false);
});
