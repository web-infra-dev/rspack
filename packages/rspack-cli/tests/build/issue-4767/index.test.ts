import path from "path";
import { run } from "../../utils/test-utils";

it("should compile nice with *.PNG resource", async () => {
	const cwd = path.resolve(__dirname);
	const { exitCode, stderr, stdout } = await run(cwd);
	expect(exitCode).toBe(0);
	expect(stderr).toBeFalsy();
	expect(stdout).toBeTruthy();
});
