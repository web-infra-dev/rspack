export default () =>
	!process.version.startsWith("v10.") && !process.version.startsWith("v12.");
