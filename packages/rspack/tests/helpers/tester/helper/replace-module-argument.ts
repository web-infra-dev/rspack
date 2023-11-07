export function replaceModuleArgument(raw: string) {
	return raw.trim().replace(/^\(function \([\w_\,\s]+\) {/, "(function () {");
}
