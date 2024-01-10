var unknown = false;

it("should not parse when logical and with `false && unknown = false`", function () {
	if (typeof __webpack_hash__ !== "string" && fail()) {
		require("fail");
	}
});

it("should not parse when logical and with `true && false = false`", function () {
	if (
		typeof __webpack_chunkname__ === "string" &&
		typeof __webpack_hash__ !== "string"
	) {
		require("fail");
	}
});

it("should not parse when logical and with `unknown && false has side effects`", function () {
	if (unknown && typeof __webpack_hash__ !== "string") {
		require("fail");
	}
});

it("should not parse when logical or with `true || unknown = true`", function () {
	if (typeof __webpack_hash__ === "string" || unknown) {
	} else {
		require("fail");
	}
});

it("should not parse when logical or with `false || true = true`", function () {
	if (
		typeof __webpack_hash__ !== "string" ||
		typeof __webpack_chunkname__ === "string"
	) {
	} else {
		require("fail");
	}
});

it("should not parse when logical or with `unknown || true has side effects`", function () {
	if (unknown || typeof __webpack_hash__ === "string") {
	} else {
		require("fail");
	}
});
