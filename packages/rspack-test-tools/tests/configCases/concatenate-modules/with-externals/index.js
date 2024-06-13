import fs from "fs";
import jq from "jquery";

it("should concatenate css", async () => {
	const content = await fs.promises.readFile(__filename, 'utf-8');
	expect(content).not.toContain(["var", "__webpack_modules__"].join(" "));
	expect(jq.version).toBe(1);
});
