function writeErrorInRed(message) {
	console.error("");
	console.error("\u001b[31m" + message + "\u001b[39m");
}

writeErrorInRed(
	`* * * * * * * * * * * * * THIS PACKAGE WAS RENAMED! * * * * * * * * * * * * * *`
);

console.error(`
IMPORTANT: This package has moved under the "@rspack/cli" NPM scope.

To learn about the Rspack project, please visit https://rspack.dev/`);

writeErrorInRed(
	`* * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * *\n`
);

process.exit(1);
