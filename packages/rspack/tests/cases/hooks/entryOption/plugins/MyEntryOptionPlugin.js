class MyEntryOptionPlugin {
	apply(compiler) {
		compiler.hooks.entryOption.tap("MyEntryOptionPlugin", (context, entry) => {
			console.log(entry, context);
		});
	}
}

module.exports = MyEntryOptionPlugin;
