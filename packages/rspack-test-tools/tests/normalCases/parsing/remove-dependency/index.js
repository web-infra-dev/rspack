it("should not generate dependency after removing by evaluation for api", function () {
	if (typeof require !== "function" /* always false */) {
		__webpack_hash__;
		require("failed");
	}
});

it("should not generate dependency after removing by evaluation for nested webpack require", function () {
	if (typeof require !== "function" /* always false */) {
		__webpack_require__.d = 1;
		function __webpack_require__() {}
		require("failed");
	}
});

it("should not generate dependency after removing by evaluation for nested webpack exports info", function () {
	if (typeof require !== "function" /* always false */) {
		__webpack_exports_info__;
		require("failed");
	}
});

it("should not generate dependency after removing by evaluation for module id", function () {
	if (typeof require !== "function" /* always false */) {
		module.id;
		require("failed");
	}
});

it("should not generate dependency after removing by evaluation for exports", function () {
	if (typeof require !== "function" /* always false */) {
		exports.aaa;
		require("failed");
	}
});

it("should not generate dependency after removing by evaluation for __filename", function () {
	if (typeof require !== "function" /* always false */) {
		__filename;
		require("failed");
	}
});

it("should not generate dependency after removing by evaluation for new URL", function () {
	if (typeof require !== "function" /* always false */) {
		new URL("file://abc");
		require("failed");
	}
});

it("should not generate dependency after removing by evaluation for import meta", function () {
	if (typeof require !== "function" /* always false */) {
		import.meta;
		require("failed");
	}
});

it("should not generate dependency after removing by evaluation for import()", function () {
	if (typeof require !== "function" /* always false */) {
		import("failed");
	}
});

it("should not generate dependency after removing by evaluation for new Worker()", function () {
	if (typeof require !== "function" /* always false */) {
		new Worker("aaa");
		require("failed");
	}
});
