it("reads the current options and calls option functions on the compiler thread", () => {
	expect(require("./lib")).toEqual({ inWorker: true, value: "at execution" });
});
