import * as Clipanion from 'clipanion';

export interface CommonCLIOptions {
  config?: string,
}

export class CommonCommand extends Clipanion.Command {
  config = Clipanion.Option.String('-c, --config')

  execute(): Promise<number | void> {
    throw new Error('Method not implemented.');
  }
}