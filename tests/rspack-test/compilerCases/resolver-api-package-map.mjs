import path from "node:path";

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
export default {
  description: "should resolve module paths with a package map",
  options(context) {
    return {
      entry: "./a.js",
    }
  },
  async check({ compiler }) {
    const { ResolverFactory } = compiler.rspack.experiments.resolver;
    const root = path.resolve(import.meta.dirname, "../configCases/resolve/package-map");
    const withPackageMap = new ResolverFactory({
      packageMap: path.join(root, "package-map.json"),
    });
    const withoutPackageMap = new ResolverFactory({});

    expect(withPackageMap.sync(root, "dep").path).toBe(
      path.join(root, "store/dep-1.0.0/index.js")
    );
    expect((await withPackageMap.async(root, "dep/sub")).path).toBe(
      path.join(root, "store/dep-1.0.0/lib/sub.js")
    );
    // Each package resolves its own dependencies.
    expect(withPackageMap.sync(path.join(root, "store/dep-1.0.0"), "shared").path).toBe(
      path.join(root, "store/shared-2.0.0/index.js")
    );
    expect(withPackageMap.sync(root, "hidden").error).toContain("Cannot find module 'hidden'");
    // A resolver created from another one keeps the package map unless it is replaced.
    expect(withPackageMap.cloneWithOptions({ packageMap: path.join(root, "package-map.json") }).sync(root, "dep").path).toBe(
      path.join(root, "store/dep-1.0.0/index.js")
    );

    expect(withoutPackageMap.sync(root, "dep").path).toBe(
      path.join(root, "node_modules/dep/index.js")
    );
    expect(withoutPackageMap.sync(root, "hidden").path).toBe(
      path.join(root, "node_modules/hidden/index.js")
    );
  }
}
