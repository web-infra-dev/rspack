import type { RegisterJsTaps } from "@rspack/binding";
import type { Compiler } from "../Compiler";
import { createHtmlPluginHooksRegisters } from "../builtin-plugin/html-plugin/taps";
import { createCompilationHooksRegisters } from "./compilation";
import { createCompilerHooksRegisters } from "./compiler";
import { createContextModuleFactoryHooksRegisters } from "./contextModuleFactory";
import { createJavaScriptModulesHooksRegisters } from "./javascriptModules";
import { createNormalModuleFactoryHooksRegisters } from "./normalModuleFactory";

export function createHooksRegisters(compiler: Compiler): RegisterJsTaps {
	const ref = new WeakRef(compiler);
	const getCompiler = () => ref.deref()!;
	const createTap =
		compiler.__internal__create_hook_register_taps.bind(compiler);
	const createMapTap =
		compiler.__internal__create_hook_map_register_taps.bind(compiler);
	return {
		...createCompilerHooksRegisters(getCompiler, createTap, createMapTap),
		...createCompilationHooksRegisters(getCompiler, createTap, createMapTap),
		...createNormalModuleFactoryHooksRegisters(
			getCompiler,
			createTap,
			createMapTap
		),
		...createContextModuleFactoryHooksRegisters(
			getCompiler,
			createTap,
			createMapTap
		),
		...createJavaScriptModulesHooksRegisters(
			getCompiler,
			createTap,
			createMapTap
		),
		...createHtmlPluginHooksRegisters(getCompiler, createTap, createMapTap)
	};
}
