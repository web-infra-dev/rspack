/** @typedef {"info" | "warning" | "error"} LogLevel */

/** @type {LogLevel} */
var logLevel = 'info';

function dummy() {}

/**
 * @param {LogLevel} level log level
 * @returns {boolean} true, if should log
 */
function shouldLog(level) {
  var shouldLog =
    (logLevel === 'info' && level === 'info') ||
    (['info', 'warning'].indexOf(logLevel) >= 0 && level === 'warning') ||
    (['info', 'warning', 'error'].indexOf(logLevel) >= 0 && level === 'error');
  return shouldLog;
}

/**
 * @param {(msg?: string) => void} logFn log function
 * @returns {(level: LogLevel, msg?: string) => void} function that logs when log level is sufficient
 */
function logGroup(logFn) {
  return function (level, msg) {
    if (shouldLog(level)) {
      logFn(msg);
    }
  };
}

/**
 * @param {LogLevel} level log level
 * @param {string|Error} msg message
 */
function log(level, msg) {
  if (shouldLog(level)) {
    if (level === 'info') {
      console.log(msg);
    } else if (level === 'warning') {
      console.warn(msg);
    } else if (level === 'error') {
      console.error(msg);
    }
  }
}

var group = console.group || dummy;
var groupCollapsed = console.groupCollapsed || dummy;
var groupEnd = console.groupEnd || dummy;

export var group = logGroup(group);

export var groupCollapsed = logGroup(groupCollapsed);

export var groupEnd = logGroup(groupEnd);

/**
 * @param {LogLevel} level log level
 */
export var setLogLevel = function (level) {
  logLevel = level;
};

/**
 * @param {Error} err error
 * @returns {string} formatted error
 */
export var formatError = function (err) {
  var message = err.message;
  var stack = err.stack;
  if (!stack) {
    return message;
  } else if (stack.indexOf(message) < 0) {
    return message + '\n' + stack;
  } else {
    return stack;
  }
};

log.group = group;
log.groupCollapsed = groupCollapsed;
log.groupEnd = groupEnd;
log.setLogLevel = setLogLevel;
log.formatError = formatError;

// TODO: remove default export when rspack-dev-server refactored
export default log;
export { log };
