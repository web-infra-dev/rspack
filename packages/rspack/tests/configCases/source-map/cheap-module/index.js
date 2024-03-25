const fs = require("fs");
const sourceMap = require("source-map");
require("./index.scss");

it("should only map original lines if cheap module options is used", async () => {
	const source = fs.readFileSync(__dirname + "/bundle0.css.map", "utf-8");
	const map = JSON.parse(source);
	expect(map.sources).toContain("webpack:///./index.scss");
	expect(map.file).toEqual("bundle0.css");
	expect(map.sourcesContent[0]).toContain("$backgroundColor");
	const consumer = await new sourceMap.SourceMapConsumer(map);
	consumer.eachMapping(m => {
		expect(m.generatedColumn).toBe(0);
		expect(m.originalColumn).toBe(0);
	});
});
