import {
	ns as nsWith,
	default as defWith,
	data as dataWith
} from "./reexport-with-default.mjs";
import {
	ns as nsWithout,
	default as defWithout,
	data as dataWithout
} from "./reexport-without-default.mjs";

it("should keep the namespace default apart from the default property of a CommonJs module", function () {
	expect(nsWith.default).toEqual({ data: "ok", default: "default" });
	expect(nsWith.default.default).toBe("default");
	expect(nsWith.data).toBe("ok");
	expect(defWith).toEqual({ data: "ok", default: "default" });
	expect(defWith.default).toBe("default");
	expect(dataWith).toBe("ok");
});

it("should provide the namespace default of a CommonJs module without a default property", function () {
	expect(nsWithout.default).toEqual({ data: "ok", other: "other" });
	expect(nsWithout.default.other).toBe("other");
	expect(nsWithout.data).toBe("ok");
	expect(defWithout).toEqual({ data: "ok", other: "other" });
	expect(dataWithout).toBe("ok");
});
