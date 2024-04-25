import a from "./a";

const obj = {
	get [a]() {
		return "success";
	}
};

it("should compile", () => {
	expect(obj[a]).toBe("success");
});
