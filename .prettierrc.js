module.exports = {
	printWidth: 80,
	useTabs: true,
	tabWidth: 2,
	trailingComma: "none",
	arrowParens: "avoid",
	overrides: [
		{
			files: "*.json",
			options: {
				parser: "json",
				useTabs: false
			}
		},
		{
			files: "*.ts",
			options: {
				parser: "typescript"
			}
		},
		{
			files: "website/**",
			options: {
				printWidth: 80,
				singleQuote: true,
				trailingComma: "all",
				useTabs: false
			}
		}
	]
};
