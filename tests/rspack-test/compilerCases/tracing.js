
defineCompileCase(Utils.basename(__filename), {
	description: "support call register global trace and cleanup global trace multi times",
	async check({ compiler }) {
		await compiler.rspack.experiments.globalTrace.register('info', 'logger', 'stdout');
		await compiler.rspack.experiments.globalTrace.register('info', 'logger', 'stdout');
		await compiler.rspack.experiments.globalTrace.cleanup();
		await compiler.rspack.experiments.globalTrace.cleanup();
		await compiler.rspack.experiments.globalTrace.register('info', 'logger', 'stdout');
		await compiler.rspack.experiments.globalTrace.cleanup();
	}
});
