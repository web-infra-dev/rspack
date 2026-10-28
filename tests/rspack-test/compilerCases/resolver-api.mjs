import path from "node:path";

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
export default {
  description: "should successfully resolve module paths",
  options(context) {
    return {
      entry: "./a.js",
    }
  },
  async check({ compiler }) {
    let resolver = compiler.rspack.experiments.resolver;

    async function testResolver(path, request) {
      expect(resolver.sync(path, request).path).toBeDefined();
      expect((await resolver.async(path, request)).path).toBeDefined();

      const customResolver = new resolver.ResolverFactory({});
      expect(customResolver.sync(path, request).path).toBeDefined();
      expect((await customResolver.async(path, request)).path).toBeDefined();
    }

    await Promise.all([
      testResolver(path.resolve(import.meta.dirname, "../fixtures"), "./main1.js"),
      testResolver(".", "react")
    ]);
  }
}
