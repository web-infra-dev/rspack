import { Command } from 'commander';
import { createServer } from 'rspack-dev-server';
import { Rspack } from "../server";
import fs from 'fs';

const program = new Command();

program.command('build')
  .description('Rspack build cli')
  .argument('<config-json>', 'rspack config json file path')
  .action(async (options) => {
    const rspack = new Rspack(JSON.parse(fs.readFileSync(options).toString()));
    const stats = await rspack.build();
    console.log(stats);
  });

program.command('dev')
  .description('Rspack build cli')
  .argument('<config-json>', 'rspack config json file path')
  .action(async (configFilePath) => {
    const rspack = new Rspack(JSON.parse(fs.readFileSync(configFilePath).toString()));
    const {
      options: {
        dev: {
          port = 8080
        } = {}
      } = {}
    } = rspack;

    await rspack.build();
    const server = await createServer(rspack.options);
    server.listen(port, () => console.log(`Server listening on port: ${port}`));
  });

program.parse();