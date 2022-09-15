import assert from "assert";
import { test, suite } from "uvu";
import { runCase } from "./case.template";

test("case-runner", () => {
	runCase({
		name: "basic"
	});
});

test.run();
