import { Command } from 'commander';
import { createServer } from 'rspack-dev-server';
import { Rspack } from '..';
import fs from 'fs';
import { build } from '../build';

const program = new Command();

program
  .option('--env', 'env')
  .command('build', {
    isDefault: true,
  })
  .description('Rspack build cli')

  .argument('<config-file>', 'rspack config  file path')
  .action(async (configPath) => {
    const config = require(configPath);
    const stats = await build(config);
    console.log(stats);
  });

program
  .command('dev')
  .description('Rspack build cli')
  .argument('<config-file>', 'rspack config file path')
  .action(async (configPath) => {
    const config = require(configPath);
    const rspack = new Rspack(config);
    const { options: { dev: { port = 8080 } = {} } = {} } = rspack;
    await rspack.build();
    const server = await createServer(rspack.options);
    server.listen(port, () => console.log(`Server listening on port: ${port}`));
  });

program.parse();
