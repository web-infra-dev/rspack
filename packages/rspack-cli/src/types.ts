import type { RspackCLI } from './cli';

export type { Configuration } from '@rspack/core';

export type LogHandler = (value: any) => void;

export interface RspackCLIColors {
  isColorSupported: boolean;
  red(text: string): string;
  yellow(text: string): string;
  cyan(text: string): string;
  green(text: string): string;
}

export interface RspackCLILogger {
  error: LogHandler;
  warn: LogHandler;
  info: LogHandler;
  success: LogHandler;
  log: LogHandler;
  raw: LogHandler;
}

export interface RspackCommand {
  apply(cli: RspackCLI): Promise<void>;
}
