export enum RequestType {
	AddDependency = "AddDependency",
	AddContextDependency = "AddContextDependency",
	AddMissingDependency = "AddMissingDependency",
	AddBuildDependency = "AddBuildDependency",
	GetDependencies = "GetDependencies",
	GetContextDependencies = "GetContextDependencies",
	GetMissingDependencies = "GetMissingDependencies",
	ClearDependencies = "ClearDependencies",
	Resolve = "Resolve",
	GetResolve = "GetResolve",
	GetLogger = "GetLogger",
	EmitError = "EmitError",
	EmitWarning = "EmitWarning",
	EmitFile = "EmitFile",
	EmitDiagnostic = "EmitDiagnostic",
	SetCacheable = "SetCacheable",
	ImportModule = "ImportModule",
	UpdateLoaderObjects = "UpdateLoaderObjects",
	CompilationGetPath = "CompilationGetPath",
	CompilationGetPathWithInfo = "CompilationGetPathWithInfo",
	CompilationGetAssetPath = "CompilationGetAssetPath",
	CompilationGetAssetPathWithInfo = "CompilationGetAssetPathWithInfo"
}

export type HandleIncomingRequest = (
	requestType: RequestType,
	...args: any[]
) => Promise<any> | any;

export async function run() {
	throw new Error("Not support browser");
}
