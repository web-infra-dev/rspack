import { Command as CommanderCommand } from "commander";

const Command = "entry command";
const commander = "entry local";

export function createCommand(name) {
	return new CommanderCommand(name);
}

it("should link a destructured direct export", () => {
	const command = createCommand("build");

	expect(command.name()).toBe("build");
	expect(Command).toBe("entry command");
	expect(commander).toBe("entry local");
});
