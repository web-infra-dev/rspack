import { rspack } from "@rspack/core";
import { createFsFromVolume, Volume } from "memfs";
import { closeCompiler, runCompiler } from "@rspack/test-tools/helper/lifecycle";

export default async function run() {
  const compiler = rspack({
    context: import.meta.dirname,
    mode: "development",
    entry: "./entry.js",
    output: { path: "/", filename: "bundle.js" },
  });
  compiler.outputFileSystem = createFsFromVolume(new Volume());

  try {
    const firstStats = await runCompiler(compiler);
    const dependency =
      firstStats.compilation.entries.values().next().value.dependencies[0];
    const type = dependency.type;
    const request = dependency.request;

    await runCompiler(compiler);

    expect(dependency.type).toBe(type);
    expect(dependency.request).toBe(request);
    expect(() => dependency._parentModule).toThrow(
      "Unable to access compilation",
    );
  } finally {
    await closeCompiler(compiler);
  }
}
