import { cleanUp } from '../ErrorHelpers';
import WebpackError from '../lib/WebpackError';

const createMessage = (
  err: Error,
  type: 'Error' | 'Warning',
  from?: string,
) => {
  let message = `Module ${type}${from ? ` (from ${from}):\n` : ': '}`;

  if (err && typeof err === 'object' && err.message) {
    message += err.message;
  } else if (err) {
    message += err;
  }

  return message;
};

const getErrorDetails = (err: Error) =>
  err && typeof err === 'object' && err.stack
    ? cleanUp(err.stack, err.name, err.message)
    : undefined;

export class ModuleError extends WebpackError {
  error?: Error;

  constructor(err: Error, { from }: { from?: string } = {}) {
    super(createMessage(err, 'Error', from));
    this.name = 'ModuleError';
    this.error = err;
    this.details = getErrorDetails(err);
  }
}

export class ModuleWarning extends WebpackError {
  error?: Error;

  constructor(err: Error, { from }: { from?: string } = {}) {
    super(createMessage(err, 'Warning', from));
    this.name = 'ModuleWarning';
    this.error = err;
    this.details = getErrorDetails(err);
  }
}
