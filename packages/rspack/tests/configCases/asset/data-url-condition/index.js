import fs from "fs";
import path from "path";

import LARGE from "./large.png";

it("should inline the content if `rule.type` is sat to `asset` and the max size of the condition doesn't exceeds the `dataUrlCondition`", () => {
	const png = fs.readFileSync(path.join(__dirname, "../large.png"));
	expect(png.length < 100 * 1024).toBeTruthy();
	expect(LARGE.startsWith("data:image/png;base64,")).toBeTruthy();
});
