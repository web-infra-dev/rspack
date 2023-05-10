#!/usr/bin/env zx

import "zx/globals";
import { Command } from "commander";

process.env.FORCE_COLOR = 3; // Fix zx losing color output in subprocesses

const program = new Command();

program
  .name("Rspack Development CLI")
  .description("CLI for development of Rspack")
  .showHelpAfterError(true)
  .showSuggestionAfterError(true);

// x install
program
  .command("install")
  .alias("i")
  .description("install all dependencies")
  .action(async function () {
    await $`pnpm install`;
  });

// x clean
let cleanCommand = program
  .command("clean")
  .description("clean target/ directory");

// x clean all
cleanCommand.command("all").action(async function () {
  await $`./x clean rust`;
});

// x clean rust
cleanCommand
  .command("rust")
  .description("clean target/ directory")
  .action(async function () {
    await $`cargo clean`;
  });

// x build
const buildCommand = program.command("build").alias("b").description("build");

// x build binding
buildCommand.command("binding").action(async function () {
  await $`pnpm --filter @rspack/binding build:debug`;
});

let argv = process.argv.slice(2); // remove the `node` and script call
if (argv[0] && /x.mjs/.test(argv[0])) {
  // Called from `zx x.mjs`
  argv = argv.slice(1);
}
program.parse(argv, { from: "user" });
