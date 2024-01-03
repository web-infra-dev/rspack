import path from "path";
import { run } from "../../utils/test-utils";

it("should not print the warning for child compiler", async () => {
	const cwd = path.resolve(__dirname, "./child");
	const { exitCode, stderr } = await run(
		cwd,
		[],
		{},
		{
			RSPACK_DEP_WARNINGS: true
		}
	);
	expect(exitCode).toBe(0);
	expect(stderr).not.toContain(
		'\'builtins.banner = {"banner":""}\' has been deprecated'
	);
	expect(stderr).not.toContain(
		`'builtins.decorator = {"legacy":true,"emitMetadata":true}' onl`
	);
});

it("should print the warning for root compiler", async () => {
	const cwd = path.resolve(__dirname, "./root");
	const { exitCode, stderr } = await run(
		cwd,
		[],
		{},
		{
			RSPACK_DEP_WARNINGS: true
		}
	);
	expect(exitCode).toBe(0);
	expect(stderr).toContain(
		'\'builtins.banner = {"banner":""}\' has been deprecated'
	);
});
