import {
	Command as CommandA,
	createCommand as createCommandA
} from "./wrapper-a";
import {
	Command as CommandB,
	createCommand as createCommandB
} from "./wrapper-b";

export { CommandA, CommandB, createCommandA, createCommandB };

const Command = "consumer command";

it("should deconflict destructured exports from multiple wrappers", () => {
	const commandA = new CommandA("build");
	const commandB = new CommandB("build");

	expect(commandA.name).toBe("a:build");
	expect(commandB.name).toBe("b:build");
	expect(createCommandA("serve").name).toBe("a:serve");
	expect(createCommandB("serve").name).toBe("b:serve");
	expect(Command).toBe("consumer command");
});
