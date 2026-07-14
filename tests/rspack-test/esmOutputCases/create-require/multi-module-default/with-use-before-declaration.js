import { createRequire } from "node:module";

function invokeBeforeDeferredRequireDeclaration() {
	deferredPreInitializationRequire("./dep.js");
}

let deferredFunctionPreInitializationThrows = false;
try {
	invokeBeforeDeferredRequireDeclaration();
} catch {
	deferredFunctionPreInitializationThrows = true;
}
const deferredPreInitializationRequire = createRequire(import.meta.url);

function readUnknownMember() {
	return require.a;
}

function loadBuiltin() {
	return typeof require("path").join;
}

function loadLocalModule() {
	return require("./dep.js");
}

function loadHandledBeforeDeclaration() {
	return handledBeforeDeclarationRequire("./dep.js");
}

function loadMutableBeforeDeclaration() {
	return mutableBeforeDeclarationRequire("./dep.js");
}

const loadBoundBeforeDeclaration = function () {
	return boundBeforeDeclarationRequire("./dep.js");
}.bind(null);

let assignedRequire;
function readAssignedUnknownMember() {
	return assignedRequire.a;
}

const require = createRequire(import.meta.url);
const boundBeforeDeclarationRequire = createRequire(import.meta.url);
const handledBeforeDeclarationRequire = createRequire(import.meta.url);
let mutableBeforeDeclarationRequire = createRequire(import.meta.url);
mutableBeforeDeclarationRequire = request => request;
let mutableAfterDeclarationRequire = createRequire(import.meta.url);
function loadMutableAfterDeclaration() {
	return mutableAfterDeclarationRequire("./dep.js");
}
mutableAfterDeclarationRequire = request => request;
const assignmentSourceRequire = createRequire(import.meta.url);
assignedRequire = assignmentSourceRequire;

const createRequireAlias = createRequire;
let mutableCreateRequireAlias = createRequire;
function loadMutableCreateRequireAlias() {
	return mutableCreateRequireAlias(import.meta.url)("./dep.js");
}
mutableCreateRequireAlias = () => request => request;
let mutableAssignedRequire;
const mutableAssignmentSourceRequire = createRequire(import.meta.url);
mutableAssignedRequire = mutableAssignmentSourceRequire;
function loadMutableAssignedRequire() {
	return mutableAssignedRequire("./dep.js");
}
mutableAssignedRequire = request => request;
let mutableAssignedCreateRequire;
mutableAssignedCreateRequire = createRequire;
function loadMutableAssignedCreateRequire() {
	return mutableAssignedCreateRequire(import.meta.url)("./dep.js");
}
mutableAssignedCreateRequire = () => request => request;
function readAliasedCalleeUnknownMember() {
	return aliasedCalleeRequire.a;
}
function loadAliasedCalleeLocalModule() {
	return aliasedCalleeRequire("./dep.js");
}
const aliasedCalleeRequire = createRequireAlias(import.meta.url);
const handledAliasedCalleeRequire = (0, createRequireAlias)(import.meta.url);

export const beforeAliasedCalleeUnknownMember =
	readAliasedCalleeUnknownMember();
export const beforeAliasedCalleeRequired = loadAliasedCalleeLocalModule();
export const handledAliasedCalleeRequired =
	handledAliasedCalleeRequire("./dep.js");
export const beforeAssignmentUnknownMember = readAssignedUnknownMember();
export const beforeDeclarationBuiltinJoinType = loadBuiltin();
export const boundBeforeDeclarationRequired = loadBoundBeforeDeclaration();
export const beforeDeclarationRequired = loadLocalModule();
export const handledBeforeDeclarationRequired = loadHandledBeforeDeclaration();
export const mutableAfterDeclarationResult = loadMutableAfterDeclaration();
export const mutableBeforeDeclarationResult = loadMutableBeforeDeclaration();
export const mutableCreateRequireAliasResult = loadMutableCreateRequireAlias();
export const mutableAssignedRequireResult = loadMutableAssignedRequire();
export const mutableAssignedCreateRequireResult =
	loadMutableAssignedCreateRequire();
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
	deferredFunctionPreInitializationThrows,
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
	})(),
	(() => {
		try {
			localCreateRequireChain(import.meta.url)("./dep.js");
			const localCreateRequireChain = createRequire;
			return false;
		} catch {
			return true;
		}
	})(),
	(() => {
		try {
			(() => immediateRequire("./dep.js"))();
			const immediateRequire = createRequire(import.meta.url);
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
export const preInitializationImmediateExecutionThrows = [
	(() => {
		try {
			new (function () {
				constructorRequire("./dep.js");
			})();
			const constructorRequire = createRequire(import.meta.url);
			return false;
		} catch {
			return true;
		}
	})(),
	(() => {
		try {
			(function () {
				taggedTemplateRequire("./dep.js");
			})``;
			const taggedTemplateRequire = createRequire(import.meta.url);
			return false;
		} catch {
			return true;
		}
	})(),
	(() => {
		try {
			new (class {
				constructor() {
					classConstructorRequire("./dep.js");
				}
			})();
			const classConstructorRequire = createRequire(import.meta.url);
			return false;
		} catch {
			return true;
		}
	})(),
	(() => {
		try {
			(function ({}) {
				complexIifeRequire("./dep.js");
			})({});
			const complexIifeRequire = createRequire(import.meta.url);
			return false;
		} catch {
			return true;
		}
	})(),
	(() => {
		try {
			(function ({}) {
				complexCallRequire("./dep.js");
			}).call(null, {});
			const complexCallRequire = createRequire(import.meta.url);
			return false;
		} catch {
			return true;
		}
	})(),
	(() => {
		try {
			(function ({}) {
				complexApplyRequire("./dep.js");
			}).apply(null, [{}]);
			const complexApplyRequire = createRequire(import.meta.url);
			return false;
		} catch {
			return true;
		}
	})(),
	(() => {
		try {
			(function ({}) {
				computedCallRequire("./dep.js");
			})["call"](null, {});
			const computedCallRequire = createRequire(import.meta.url);
			return false;
		} catch {
			return true;
		}
	})(),
	(() => {
		try {
			(() => arrowCallRequire("./dep.js")).call(null);
			const arrowCallRequire = createRequire(import.meta.url);
			return false;
		} catch {
			return true;
		}
	})()
];

var logicalOrRequire;
logicalOrRequire ||= require("./dep.js");
const logicalOrValue = logicalOrRequire;
var logicalOrRequire = createRequire(import.meta.url);

var logicalNullishRequire;
logicalNullishRequire ??= require("./dep.js");
const logicalNullishValue = logicalNullishRequire;
var logicalNullishRequire = createRequire(import.meta.url);

export const preInitializationLogicalAssignmentValues = [
	logicalOrValue,
	logicalNullishValue
];
