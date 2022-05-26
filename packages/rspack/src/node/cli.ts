import path from 'path';
import fs from 'fs';
const yargs: typeof import('yargs') = require('yargs');
import { run } from './core';
import { RawOptions } from '@rspack/binding';

// build({ main: path.resolve(__dirname, '../fixtures/index.js') });

yargs
  .scriptName('rspack')
  .usage('$0 <root>')
  .command(
    'build [root]',
    'rspack build',
    (yargs) => {
      yargs.positional('root', {
        type: 'string',
        default: process.cwd(),
        describe: 'project root',
      });
    },
    (argv: any) => {
      compile({ ...argv, command: 'build' });
    }
  )
  .command(
    '$0 [root]',
    'start dev server',
    (yargs) => {
      yargs.positional('root', {
        type: 'string',
        default: process.cwd(),
        describe: 'project root',
      });
    },
    (argv: any) => {
      compile({ ...argv, command: 'dev' });
    }
  )
  .help().argv;

function compile(argv: any) {
  const root = path.resolve(process.cwd(), argv.root);
  const rspackConfigPath = path.resolve(root, 'rspack.config.json');
  let rspackConfig: RawOptions;
  if (fs.existsSync(rspackConfigPath)) {
    rspackConfig = JSON.parse(fs.readFileSync(rspackConfigPath).toString());
  } else {
    rspackConfig = {
      entries: { main: path.resolve(root, 'index.js') },
    };
  }
  rspackConfig.root ??= root;
  rspackConfig.resolve ??= {};
  rspackConfig.resolve.alias ??= {};
  rspackConfig.resolve.alias = Object.fromEntries(
    Object.entries(rspackConfig.resolve.alias).map(([key, value]) => [key, (value as string).replace('<ROOT>', root)])
  );

  run(rspackConfig, argv.command);
}
