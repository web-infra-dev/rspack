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
buildCommand
  .command("binding")
  .description("build rust binding")
  .action(async function () {
    await $`pnpm --filter @rspack/binding build:debug`;
  });

// x build js
buildCommand
  .command("js")
  .description("build js packages")
  .action(async function () {
    await $`pnpm --filter "@rspack/*" build`;
  });

// x test
const testCommand = program.command("test").alias("t").description("test");

// x test rust
testCommand
  .command("rust")
  .description("run cargo tests")
  .action(async function () {
    await $`cargo test`;
  });

// x test example
testCommand
  .command("example")
  .description("build examples")
  .action(async function () {
    await $`pnpm --filter "example-*" build`;
  });

// x test unit
testCommand
  .command("unit")
  .description("run all unit tests")
  .action(async function () {
    await $`./x build js`;
    await $`pnpm --filter "@rspack/*" test`;
  });

// x test ci
testCommand
  .command("ci")
  .description("run tests for ci")
  .action(async function () {
    await $`./x test example`;
    await $`./x test unit`;
  });

let argv = process.argv.slice(2); // remove the `node` and script call
if (argv[0] && /x.mjs/.test(argv[0])) {
  // Called from `zx x.mjs`
  argv = argv.slice(1);
}
program.parse(argv, { from: "user" });
