if (!process.env.COREPACK_ROOT) {
	console.error(
		"please ensure corepack is enabled https://pnpm.io/installation#using-corepack"
	);
	process.exit(1);
}
