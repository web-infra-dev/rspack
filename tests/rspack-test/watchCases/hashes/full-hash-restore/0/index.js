import value from "./value";

it("should restore the full hash when module content is restored without HMR", () => {
	expect(value).toBe(WATCH_STEP === "1" ? 2 : 1);
	switch (WATCH_STEP) {
		case "0":
			STATE.hash = __webpack_hash__;
			break;
		case "1":
			expect(__webpack_hash__).not.toBe(STATE.hash);
			break;
		case "2":
			expect(__webpack_hash__).toBe(STATE.hash);
			break;
	}
});
