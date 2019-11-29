const util = require('util');
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
function log(level, target, message, ...args) {
  if (!Object.prototype.hasOwnProperty.call(LEVEL_STYLES, level))
    throw new Error(`unknown level: ${level}`);
  let levelStyle = LEVEL_STYLES[level];
  let head = '';
  head += chalk.gray('[');
  head += new Date().toISOString();
  head += ' ';
  head += levelStyle(level.toUpperCase().padEnd(longestLevelNameLength, ' '));
  head += ' backend';
  if (target != null) head += `::${target}`;
  head += chalk.gray(']');
  console.log(head, message, ...args);
}

log.error = function error(target, message, ...args) {
  return log('error', target, message, ...args);
};

log.warn = function warn(target, message, ...args) {
  return log('warn', target, message, ...args);
};

log.info = function info(target, message, ...args) {
  return log('info', target, message, ...args);
};

log.debug = function debug(target, message, ...args) {
  return log('debug', target, message, ...args);
};

log.trace = function trace(target, message, ...args) {
  return log('trace', target, message, ...args);
};

process.on('uncaughtException', error => {
  log.error(null, error);
});

process.on('unhandledRejection', (reason, _promise) => {
  log.error(null, reason);
});

module.exports = log;
