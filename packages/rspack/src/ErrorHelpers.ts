const loaderFlag = 'LOADER_EXECUTION';

const cutOffByFlag = (stack: string, flag: string) => {
  const flagIndex = stack.indexOf(flag);
  if (flagIndex === -1) return stack;
  const lineStart = stack.lastIndexOf('\n', flagIndex);
  return lineStart === -1 ? '' : stack.slice(0, lineStart);
};

export const cutOffLoaderExecution = (stack: string) =>
  cutOffByFlag(stack, loaderFlag);

export const cleanUp = (
  stack: string,
  name: string,
  message: string,
): string => {
  let details = cutOffLoaderExecution(stack);
  details = cutOffMessage(stack, name, message);
  return details;
};

export const cutOffMessage = (
  stack: string,
  name: string,
  message: string,
): string => {
  const nextLine = stack.indexOf('\n');
  if (nextLine === -1) {
    return stack === message ? '' : stack;
  }
  const firstLine = stack.slice(0, nextLine);
  return firstLine === `${name}: ${message}`
    ? stack.slice(nextLine + 1)
    : stack;
};
