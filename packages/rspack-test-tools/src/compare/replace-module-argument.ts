export function replaceModuleArgument(raw: string) {
	return raw
		.trim()
		.replace(/^\(function\s?\([\w_,\s]+\)\s?{/, "(function () {");
}
