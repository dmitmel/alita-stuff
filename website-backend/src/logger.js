const chalk = require('chalk');

const LEVEL_STYLES = {
  error: chalk.red.bold,
  warn: chalk.yellow.bold,
  info: chalk.green,
  debug: chalk.blue,
  trace: chalk.magenta,
};

let longestLevelNameLength = Math.max(
  ...Object.keys(LEVEL_STYLES).map(k => k.length),
);
function log(level, message, ...args) {
  if (!Object.prototype.hasOwnProperty.call(LEVEL_STYLES, level))
    throw new Error(`unknown level: ${level}`);
  let levelStyle = LEVEL_STYLES[level];
  let head = '';
  head += chalk.gray('[');
  head += new Date().toISOString();
  head += ' ';
  head += levelStyle(level.toUpperCase().padEnd(longestLevelNameLength, ' '));
  head += chalk.gray(']');
  console.log(head, message, ...args);
}

log.error = function error(message, ...args) {
  return log('error', message, ...args);
};

log.warn = function warn(message, ...args) {
  return log('warn', message, ...args);
};

log.info = function info(message, ...args) {
  return log('info', message, ...args);
};

log.debug = function debug(message, ...args) {
  return log('debug', message, ...args);
};

log.trace = function trace(message, ...args) {
  return log('trace', message, ...args);
};

process.on('uncaughtException', error => {
  log.error('uncaughtException:', error);
});

process.on('unhandledRejection', (reason, _promise) => {
  log.error('unhandledRejection:', reason);
});

module.exports = log;
