import { defineConfig } from '@rspack/cli';
import { EnvironmentPlugin } from '@rspack/core';

process.env.AAA = 'aaa';
process.env.BBB = 'bbb';
process.env.CCC = 'ccc';
process.env.EEE = 'eee';
process.env.FFF = 'fff';
process.env.GGG = 'ggg';
process.env.III = '';

export default defineConfig([
  {
    name: 'aaa',
    plugins: [new EnvironmentPlugin('AAA')],
  },
  {
    name: 'bbbccc',
    plugins: [new EnvironmentPlugin('BBB', 'CCC')],
  },
  {
    name: 'ddd',
    plugins: [new EnvironmentPlugin('DDD')],
  },
  {
    name: 'eeefff',
    plugins: [new EnvironmentPlugin(['EEE', 'FFF'])],
  },
  {
    name: 'ggghhh',
    plugins: [
      new EnvironmentPlugin({
        GGG: 'ggg-default',
        HHH: 'hhh',
      }),
    ],
  },
  {
    name: 'iii',
    plugins: [new EnvironmentPlugin('III')],
  },
]);
