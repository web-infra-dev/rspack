module.exports = {
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
				singleQuote: true,
				trailingComma: "all",
				useTabs: false
			}
		},
		{
			files: "website/**",
			options: {
				singleQuote: true,
				trailingComma: "all",
				useTabs: false
			}
		}
	]
};
