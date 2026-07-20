import {
	Command as CommandA,
	createCommand as createCommandA
} from "./wrapper-a";
import {
	Command as CommandB,
	createCommand as createCommandB
} from "./wrapper-b";

it("deconflicts destructured bindings from concatenated modules", () => {
	const commandA = createCommandA("test");
	const commandB = createCommandB("test");

	expect(commandA).toBeInstanceOf(CommandA);
	expect(commandA.name).toBe("a:test");
	expect(commandB).toBeInstanceOf(CommandB);
	expect(commandB.name).toBe("b:test");

	const source = require("fs").readFileSync(__filename, "utf-8");
	const declarations = source.match(
		/^const \{ .* \} = api_[ab]_namespaceObject;$/gm
	);

	/*********** DO NOT MATCH BELOW THIS LINE ***********/

	expect(declarations).toHaveLength(2);
	const bindings = declarations.flatMap(declaration =>
		Array.from(declaration.matchAll(/:\s*([A-Za-z_$][\w$]*)/g), match => match[1])
	);
	expect(bindings).toHaveLength(4);
	expect(bindings.slice(0, 2)).toEqual(["createCommand", "Command"]);
	expect(bindings[2]).not.toBe("createCommand");
	expect(bindings[3]).not.toBe("Command");
	expect(new Set(bindings).size).toBe(4);
});
