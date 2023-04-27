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

// x build
const buildCommand = program.command("build").alias("b").description("build");

// x build binding
buildCommand.command("binding").action(async function () {
  await $`pnpm --filter @rspack/binding build:debug`;
});

program.parse(process.argv.slice(3), { from: "user" });
