import fs from 'node:fs';
import type { Readable } from 'node:stream';
import type {
  MultiStats,
  MultiStatsOptions,
  Stats,
  StatsOptions,
  StatsValue,
} from '@rspack/core';
import type { RspackCLI } from '../cli';
import type { RspackCommand } from '../types';
import {
  type CommonOptionsForBuildAndServe,
  commonOptions,
  commonOptionsForBuildAndServe,
  normalizeCommonOptions,
  setDefaultNodeEnv,
} from '../utils/options';

type BuildOptions = CommonOptionsForBuildAndServe & {
  json?: boolean | string;
};

const defaultStatsOptions = {
  all: false,
  errors: true,
  warnings: true,
  moduleTrace: true,
  timings: true,
} satisfies StatsOptions;

function applyDefaultStatsOptions(stats: StatsValue | undefined): StatsOptions {
  const options: StatsOptions =
    typeof stats === 'boolean' || typeof stats === 'string'
      ? { preset: stats }
      : (stats ?? {});

  if (options.preset !== undefined || options.all !== undefined) {
    return options;
  }

  return {
    ...defaultStatsOptions,
    ...options,
  };
}

async function runBuild(cli: RspackCLI, options: BuildOptions): Promise<void> {
  setDefaultNodeEnv(options, 'production');
  normalizeCommonOptions(options, 'build');

  const logger = cli.getLogger();
  let createJsonStringifyStream: ((value: unknown) => Readable) | undefined;

  if (options.json) {
    const stream = await import('node:stream');
    const jsonExt = await import(
      /* webpackChunkName: "json-ext" */ '@discoveryjs/json-ext'
    );
    createJsonStringifyStream = (value) =>
      stream.Readable.from(jsonExt.stringifyChunked(value));
  }

  const errorHandler = (
    error: Error | null,
    stats: Stats | MultiStats | undefined,
  ) => {
    if (error) {
      logger.error(error);
      process.exit(2);
    }

    if (stats?.hasErrors()) {
      process.exitCode = 1;
    }

    if (!compiler || !stats) {
      return;
    }

    const getStatsOptions = () => {
      if (cli.isMultipleCompiler(compiler)) {
        return {
          children: compiler.compilers.map((item) =>
            applyDefaultStatsOptions(item.options?.stats),
          ),
        } satisfies MultiStatsOptions;
      }
      return applyDefaultStatsOptions(compiler.options?.stats);
    };

    const statsOptions = getStatsOptions() as StatsOptions;

    if (options.json && createJsonStringifyStream) {
      const handleWriteError = (error: Error) => {
        logger.error(error);
        process.exit(2);
      };
      if (options.json === true) {
        createJsonStringifyStream(stats.toJson(statsOptions))
          .on('error', handleWriteError)
          .pipe(process.stdout)
          .on('error', handleWriteError)
          .on('close', () => process.stdout.write('\n'));
      } else if (typeof options.json === 'string') {
        createJsonStringifyStream(stats.toJson(statsOptions))
          .on('error', handleWriteError)
          .pipe(fs.createWriteStream(options.json))
          .on('error', handleWriteError)
          // Use stderr to logging
          .on('close', () => {
            process.stderr.write(
              `[rspack-cli] ${cli.colors.green(
                `stats are successfully stored as json to ${options.json}`,
              )}\n`,
            );
          });
      }
    } else {
      const printedStats = stats.toString(statsOptions);
      // Avoid extra empty line when `stats: 'none'`
      if (printedStats) {
        logger.raw(printedStats);
      }
    }
  };

  const userOption = await cli.buildCompilerConfig(options, 'build');
  const compiler = await cli.createCompiler(userOption, errorHandler);

  if (!compiler || cli.isWatch(compiler)) {
    return;
  }

  compiler.run((error: Error | null, stats: Stats | MultiStats | undefined) => {
    compiler.close((closeErr) => {
      if (closeErr) {
        logger.error(closeErr);
      }
      errorHandler(error, stats);
    });
  });
}

export class BuildCommand implements RspackCommand {
  apply(cli: RspackCLI): void {
    const command = cli.program
      .command('', 'run the Rspack build')
      .alias('build')
      .alias('bundle')
      .alias('b');

    commonOptionsForBuildAndServe(commonOptions(command)).option(
      '--json [path]',
      'emit stats json',
    );

    command.action(
      cli.wrapAction(async (options: BuildOptions) => {
        await runBuild(cli, options);
      }),
    );
  }
}
