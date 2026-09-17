import { EntryOptionPlugin, config } from "@rspack/core";

/**
 * Use the static method in EntryOptionPlugin to
 * apply entry option for the child compiler.
 */

export default class TestApplyEntryOptionPlugin {
  constructor(options) {
    this.options = config.getNormalizedWebpackOptions(options);
  }

  apply(compiler) {
    compiler.hooks.make.tapAsync(
      "TestApplyEntryOptionPlugin",
      (compilation, cb) => {
        const child = compilation.createChildCompiler("TestApplyEntryOptionPlugin");
        EntryOptionPlugin.applyEntryOption(child, compilation.compiler.context, this.options.entry);
        child.runAsChild(cb)
      }
    )
  }
}