const { spawn } = require("child_process");
const runCommand = async (command, args, verbose, env) => {
	const p = spawn(command, args, {
		shell: true,
		stdio: verbose ? "inherit" : "ignore",
		env: env
			? {
					...process.env,
					...env,
			  }
			: undefined,
	});
	const exitCode = await new Promise((resolve) => p.once("exit", resolve));
	if (exitCode !== 0)
		throw new Error(`${command} ${args.join(" ")} failed with ${exitCode}`);
};

const run = (command, args) => runCommand(command, args, true);

module.exports = {
	runCommand,
	run,
};
