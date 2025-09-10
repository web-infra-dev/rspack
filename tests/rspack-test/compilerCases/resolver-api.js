/** @type {import('../..').TCompilerCaseConfig} */
module.exports = {
	description: "should successfully resolve module paths",
	async check(_, compiler, __) {
		let resolver = compiler.rspack.experiments.resolver;

    async function testResolver(path, request) {
      expect(resolver.sync(path, request).path).toBeDefined();
      expect((await resolver.async(path, request)).path).toBeDefined();

      const customResolver = new resolver.ResolverFactory({});
      expect(customResolver.sync(path, request).path).toBeDefined();
      expect((await customResolver.async(path, request)).path).toBeDefined();
    }

    const path = require("node:path");
		await Promise.all([
      testResolver(path.resolve(__dirname, "../fixtures"), "./main1.js"),
      testResolver(".", "react")
		]);
	}
}
