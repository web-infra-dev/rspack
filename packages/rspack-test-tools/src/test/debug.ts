import path from "node:path";
import type { RspackOptions } from "@rspack/core";
import fs from "fs-extra";
import { stringify } from "javascript-stringify";
import stringifyConfig from "../helper/stringify-config";
import type { ITestContext } from "../type";

export function generateDebugReport(context: ITestContext) {
	const report = `
## Case Meta

${generateCaseMetaPathReport(context)}

${generateCaseMetaTestConfigReport(context)}

## Compiler Options

${generateReadConfigFileReport(context)}

${generateFinalOptionsReport(context)}

## Create Compiler

${generateCreateCompilerInstanceReport(context)}

${generateCreateCompilerSetPropertiesReport(context)}

## Build

${generateBuildMethodReport(context)}

${generateBuildErrorReport(context)}

${generateBuildWarningReport(context)}

## Run

${generateRunFindBundleReport(context)}

${generateRunGetRunnerReport(context)}

${generateRunLogsReport(context)}

${generateRunErrorsReport(context)}

  `;

	const dist = context.getDist("debug.md");
	fs.ensureDirSync(path.dirname(dist));
	fs.writeFileSync(dist, report);
}

function generateCaseMetaPathReport(context: ITestContext) {
	return `
### Case Path

- Source: ${context.getSource()}
- Dist: ${context.getDist()}
- Temp: ${context.getTemp()}
  `;
}

function generateCaseMetaTestConfigReport(context: ITestContext) {
	return `
### Test Config

\`\`\`js
// ${path.resolve(context.getSource(), "./test.config.js")}
${stringify(context.getTestConfig(), null, 2)}
\`\`\`
  `;
}

function generateReadConfigFileReport(context: ITestContext) {
	const configFileInfo = context.getValue(
		DEBUG_SCOPES.CompilerOptionsReadConfigFile
	) as { file: string; config: RspackOptions };
	if (!configFileInfo) return "";
	return `
### Read Config File

\`\`\`js
// ${configFileInfo.file}
${stringifyConfig(configFileInfo.config)}
\`\`\`
  `;
}

function generateFinalOptionsReport(context: ITestContext) {
	const finalOptions = context.getCompiler().getOptions();
	return `
### Final Options

\`\`\`js
${stringifyConfig(finalOptions)}
\`\`\`
`;
}

function generateCreateCompilerInstanceReport(context: ITestContext) {
	const instanceInfo = context.getValue(
		DEBUG_SCOPES.CreateCompilerInstance
	) as { path: string; mode: string };
	if (!instanceInfo) return "";
	return `
### Create Compiler Instance

- Rspack Path: ${instanceInfo.path}
- Callback Mode: ${instanceInfo.mode}
`;
}

function generateCreateCompilerSetPropertiesReport(context: ITestContext) {
	const setPropertiesInfo = context.getValue(
		DEBUG_SCOPES.CreateCompilerSetProperties
	) as string[];
	if (!setPropertiesInfo || setPropertiesInfo.length === 0) return "";
	return `
### Set Properties

${setPropertiesInfo.map(p => `- ${p}`).join("\n")}
`;
}

function generateBuildMethodReport(context: ITestContext) {
	const buildMethod = context.getValue(DEBUG_SCOPES.BuildMethod) as {
		method: string;
		options?: any;
	};
	if (!buildMethod) return "";
	return `
### Build Method

- Method: \`compiler.${buildMethod.method}()\`
${buildMethod.options ? `- Options:\n\`\`\`js\n${stringify(buildMethod.options, null, 2)}\n\`\`\`` : ""}
`;
}

function generateBuildErrorReport(context: ITestContext) {
	const buildError = context.getValue(DEBUG_SCOPES.BuildError) as {
		type: "fatal" | "stats";
		errors: Error[];
	};
	if (!buildError) return "";
	return `
### Build Error

type: ${buildError.type}

${buildError.errors.map(e => `\`\`\`\n// message:\n${e.message}\n// stack:\n${e.stack}\n\`\`\``).join("\n\n")}
`;
}

function generateBuildWarningReport(context: ITestContext) {
	const buildWarning = context.getValue(DEBUG_SCOPES.BuildWarning) as Error[];
	if (!buildWarning) return "";
	return `
### Build Warning

${buildWarning.map(w => `\`\`\`\n// message:\n${w.message}\n// stack:\n${w.stack}\n\`\`\``).join("\n\n")}
`;
}

function generateRunFindBundleReport(context: ITestContext) {
	const runFindBundle = context.getValue(
		DEBUG_SCOPES.RunFindBundle
	) as string[];
	if (!runFindBundle) return "";
	return `
### Find Bundle

${runFindBundle.map(b => `- ${context.getDist(b)}`).join("\n")}
`;
}

function generateRunGetRunnerReport(context: ITestContext) {
	const getRunnerInfo = context.getValue(DEBUG_SCOPES.RunGetRunner) as Record<
		string,
		{ runnerKey: string; reused: boolean; runnerType?: string }
	>;
	if (!getRunnerInfo) return "";
	return `
### Get Runner

${Object.entries(getRunnerInfo)
	.map(
		([file, info]) =>
			`- ${file}: ${info.runnerKey} (Reused: ${info.reused}, Type: \`${info.runnerType}\`)`
	)
	.join("\n")}
`;
}

function generateRunLogsReport(context: ITestContext) {
	const runLogs = context.getValue(DEBUG_SCOPES.RunLogs) as string[];
	if (!runLogs) return "";
	return `
### Run Logs

${runLogs.map(l => `- ${l}`).join("\n")}
`;
}

function generateRunErrorsReport(context: ITestContext) {
	const runErrors = context.getValue(DEBUG_SCOPES.RunErrors) as Error[];
	if (!runErrors) return "";
	return `
### Run Errors

${runErrors.map(e => `\`\`\`\n// message:\n${e.message}\n// stack:\n${e.stack}\n\`\`\``).join("\n\n")}
`;
}

export const DEBUG_SCOPES = {
	CompilerOptionsReadConfigFile: "compiler-options:read-config-file",
	CompilerOptionsFinalOptions: "compiler-options:final-options",
	CreateCompilerInstance: "create-compiler:instance",
	CreateCompilerSetProperties: "create-compiler:set-properties",
	BuildMethod: "build:method",
	BuildError: "build:error",
	BuildWarning: "build:warning",
	RunFindBundle: "run:find-bundle",
	RunGetRunner: "run:get-runner",
	RunLogs: "run:logs",
	RunErrors: "run:errors"
};
