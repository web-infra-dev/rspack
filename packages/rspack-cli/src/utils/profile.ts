/**
 * `RSPACK_PROFILE=ALL` // all trace events
 * `RSPACK_PROFILE=OVERVIEW` // overview trace events
 * `RSPACK_PROFILE=warn,tokio::net=info` // trace filter from  https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html#example-syntax
 */
import fs from 'node:fs';
import path from 'node:path';
import { rspack } from '@rspack/core';

const DEFAULT_RUST_TRACE_LAYER = 'perfetto';
const STDIO_TRACE_LAYERS = new Set(['logger', 'hotpath']);

export async function applyProfile(
  filterValue: string,
  traceLayer: string = DEFAULT_RUST_TRACE_LAYER,
  traceOutput?: string,
) {
  const { asyncExitHook } = await import('exit-hook');

  if (
    traceLayer !== 'logger' &&
    traceLayer !== 'perfetto' &&
    traceLayer !== 'hotpath'
  ) {
    throw new Error(`unsupported trace layer: ${traceLayer}`);
  }
  const timestamp = Date.now();
  const defaultOutputDir = path.resolve(
    `.rspack-profile-${timestamp}-${process.pid}`,
  );
  if (!traceOutput) {
    const defaultRustTracePerfettoOutput = path.resolve(
      defaultOutputDir,
      'rspack.pftrace',
    );
    const defaultRustTraceStdioOutput = 'stdout';

    const defaultTraceOutput = STDIO_TRACE_LAYERS.has(traceLayer)
      ? defaultRustTraceStdioOutput
      : defaultRustTracePerfettoOutput;

    traceOutput = defaultTraceOutput;
  } else if (traceOutput !== 'stdout' && traceOutput !== 'stderr') {
    // if traceOutput is not stdout or stderr, we need to ensure the directory exists
    traceOutput = path.resolve(defaultOutputDir, traceOutput);
  }

  await ensureFileDir(traceOutput);
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
