const loaderFlag = 'at LOADER_EXECUTION';

const cutOffByFlag = (stack: string, flag: string) => {
  const flagIndex = stack.indexOf(flag);
  if (flagIndex === -1) return stack;
  const lineStart = stack.lastIndexOf('\n', flagIndex);
  return lineStart === -1 ? '' : stack.slice(0, lineStart);
};

export const cutOffLoaderExecution = (stack: string) =>
  cutOffByFlag(stack, loaderFlag);

export const cleanUp = (stack: string, name: string, message: string): string =>
  cutOffMessage(cutOffLoaderExecution(stack), name, message);

export const cutOffMessage = (
  stack: string,
  name: string,
  message: string,
): string => {
  const prefix = `${name}: ${message}`;
  if (stack === prefix || stack === message || (!message && stack === name)) {
    return '';
  }
  if (stack.startsWith(`${prefix}\n`)) {
    return stack.slice(prefix.length + 1);
  }
  if (!message && stack.startsWith(`${name}\n`)) {
    return stack.slice(name.length + 1);
  }
  return stack;
};
