it("should has two char for ascii code", () => {
	const obj = {
		Д: "A",
		Å: "A",
		Ð: "D",
		Þ: "o",
		å: "a",
		ð: "d",
		þ: "o",
		𝒩: "𝒩"
	};
	expect(obj["𝒩"]).toHaveLength(2);

	const content = require("fs").readFileSync(__filename, "utf-8");
	expect(content.includes(`𝒩`)).toBeFalsy();
});
