import { createRequire } from "node:module";

function readUnknownMember() {
	return require.a;
}

function loadBuiltin() {
	return typeof require("path").join;
}

let assignedRequire;
function readAssignedUnknownMember() {
	return assignedRequire.a;
}

const require = createRequire(import.meta.url);
const assignmentSourceRequire = createRequire(import.meta.url);
assignedRequire = assignmentSourceRequire;

const createRequireAlias = createRequire;
function readAliasedCalleeUnknownMember() {
	return aliasedCalleeRequire.a;
}
const aliasedCalleeRequire = createRequireAlias(import.meta.url);
const handledAliasedCalleeRequire = (0, createRequireAlias)(import.meta.url);

export const beforeAliasedCalleeUnknownMember =
	readAliasedCalleeUnknownMember();
export const handledAliasedCalleeRequired =
	handledAliasedCalleeRequire("./dep.js");
export const beforeAssignmentUnknownMember = readAssignedUnknownMember();
export const beforeDeclarationBuiltinJoinType = loadBuiltin();
export const beforeDeclarationUnknownMember = readUnknownMember();
export const shadowedCalleeThrows = (() => {
	try {
		const staleRequire = createRequire(import.meta.url);
		const createRequire = () => () => undefined;
		return false;
	} catch {
		return true;
	}
})();
export const preInitializationThrows = [
	(() => {
		try {
			lexicalRequire("path");
			const lexicalRequire = createRequire(import.meta.url);
			return false;
		} catch {
			return true;
		}
	})(),
	(() => {
		try {
			hoistedRequire("path");
			var hoistedRequire = createRequire(import.meta.url);
			return false;
		} catch {
			return true;
		}
	})(),
	(() => {
		try {
			localCreateRequire(import.meta.url);
			const localCreateRequire = createRequire;
			return false;
		} catch {
			return true;
		}
	})()
];
export const preInitializationAliasThrows = [
	(() => {
		try {
			var declaratorAlias = declaratorRequire,
				declaratorRequire = createRequire(import.meta.url);
			declaratorAlias("./declarator-alias-must-not-be-bundled.js");
			return false;
		} catch {
			return true;
		}
	})(),
	(() => {
		try {
			var assignmentAlias;
			assignmentAlias = assignmentRequire;
			var assignmentRequire = createRequire(import.meta.url);
			assignmentAlias("./assignment-alias-must-not-be-bundled.js");
			return false;
		} catch {
			return true;
		}
	})()
];
