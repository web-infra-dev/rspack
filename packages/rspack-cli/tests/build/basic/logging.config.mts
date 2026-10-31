import { defineConfig } from '../../../dist/index.js';
import type { Compiler } from '@rspack/core';
import path from 'node:path';

const COMPILATION_LOG = 'compilation logger warning';
const explicitLogging = process.env.EXPLICIT_LOGGING === 'true';
const partialStats = process.env.PARTIAL_STATS === 'true';
const statsPreset = process.env.STATS_PRESET;
const allStats = process.env.ALL_STATS === 'true';

export default defineConfig({
  entry: './src/index.js',
  output: {
    path: path.resolve(import.meta.dirname, 'dist/logging'),
  },
  ...(explicitLogging
    ? {
        stats: {
          all: false,
          logging: 'warn',
        },
      }
    : statsPreset
      ? {
          stats:
            statsPreset === 'object'
              ? {
                  preset: 'verbose',
                }
              : 'verbose',
        }
      : allStats
        ? {
            stats: {
              all: true,
            },
          }
        : partialStats
          ? {
              stats: {
                timings: false,
              },
            }
          : {}),
  plugins: [
    {
      apply(compiler: Compiler) {
        compiler.hooks.thisCompilation.tap(
          'LoggingDefaultsPlugin',
          (compilation) => {
            compilation
              .getLogger('LoggingDefaultsPlugin')
              .warn(COMPILATION_LOG);
          },
        );
      },
    },
  ],
});
