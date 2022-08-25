import * as Clipanion from 'clipanion';
import DevCommand from './dev';

export function cli(args = process.argv.slice(2)) {
  const cli = new Clipanion.Cli();
  cli.register(DevCommand);
  cli.run(args);
}