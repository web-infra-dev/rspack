/**
 * `RSPACK_PROFILE=ALL` // all trace events
 * `RSPACK_PROFILE=OVERVIEW` // overview trace events
 * `RSPACK_PROFILE=warn,tokio::net=info` // trace filter from  https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html#example-syntax
 */
import fs from 'node:fs';
import path from 'node:path';
import { rspack } from '@rspack/core';

const DEFAULT_RUST_TRACE_LAYER = 'logger';
const DEFAULT_RUST_TRACE_LOGGER_OUTPUT = 'rspack.log';
const DEFAULT_RUST_TRACE_PERFETTO_OUTPUT = 'rspack.pftrace';

function isTerminalTraceOutput(output: string) {
  return output === 'stdout' || output === 'stderr';
}

export async function applyProfile(
  filterValue: string,
  traceLayer: string = DEFAULT_RUST_TRACE_LAYER,
  traceOutput?: string,
) {
  const { asyncExitHook } = await import(
    /* webpackChunkName: "exit-hook" */ 'exit-hook'
  );

  if (traceLayer !== 'logger' && traceLayer !== 'perfetto') {
    throw new Error(`unsupported trace layer: ${traceLayer}`);
  }
  if (
    traceOutput &&
    traceLayer === 'perfetto' &&
    isTerminalTraceOutput(traceOutput)
  ) {
    throw new Error(
      'RSPACK_TRACE_OUTPUT=stdout|stderr is only supported for the logger trace layer. The perfetto trace layer requires a file path.',
    );
  }

  const timestamp = Date.now();
  const defaultOutputDir = path.resolve(
    `.rspack-profile-${timestamp}-${process.pid}`,
  );
  if (!traceOutput) {
    const defaultRustTraceOutput =
      traceLayer === 'perfetto'
        ? DEFAULT_RUST_TRACE_PERFETTO_OUTPUT
        : DEFAULT_RUST_TRACE_LOGGER_OUTPUT;

    traceOutput = path.resolve(defaultOutputDir, defaultRustTraceOutput);
  } else if (!isTerminalTraceOutput(traceOutput)) {
    // if traceOutput is not stdout or stderr, we need to ensure the directory exists
    traceOutput = path.resolve(defaultOutputDir, traceOutput);
  }

  if (!isTerminalTraceOutput(traceOutput)) {
    await ensureFileDir(traceOutput);
  }
  await rspack.experiments.globalTrace.register(
    filterValue,
    traceLayer,
    traceOutput,
  );
  asyncExitHook(rspack.experiments.globalTrace.cleanup, {
    wait: 500,
  });
}

async function ensureFileDir(outputFilePath: string) {
  const dir = path.dirname(outputFilePath);
  await fs.promises.mkdir(dir, { recursive: true });
}
