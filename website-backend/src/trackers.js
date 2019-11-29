const fs = require('fs');
const path = require('path');
const typeCheck = require('./utils/typeCheck');
const log = require('./logger');

const TRACKERS_DIRECTORY = path.join(__dirname, 'trackers');
const TRACKERS = new Map(
  fs
    .readdirSync(TRACKERS_DIRECTORY)
    .filter(filename => path.extname(filename) === '.js')
    .map(filename => {
      /* eslint-disable-next-line global-require */
      let { name, createFetcher } = require(path.join(
        TRACKERS_DIRECTORY,
        filename,
      ));
      typeCheck.assert(name, 'name', 'String');
      typeCheck.assert(createFetcher, 'createFetcher', 'Function');

      return [name, createFetcher];
    }),
);

module.exports = function start(trackerConfigs) {
  typeCheck.assert(trackerConfigs, 'trackerConfigs', 'Array');

  let intervalIds = trackerConfigs.map((trackerConfig, index) => {
    typeCheck.assert(trackerConfig, 'trackerConfig', 'Object');
    let { type, requestInterval, options } = trackerConfig;
    typeCheck.assert(type, 'type', 'String');
    typeCheck.assert(requestInterval, 'requestInterval', 'Number');
    typeCheck.assert(options, 'options', 'Object');

    if (!TRACKERS.has(type)) throw new Error(`unknown tracker: ${type}`);
    let createFetcher = TRACKERS.get(type);
    let fetcher = createFetcher(options);
    typeCheck.assert(fetcher, 'fetcher', 'Function');

    log.info(
      'trackers',
      `registered tracker #${index}:\n  type: ${type}\n  request interval: ${requestInterval} seconds\n  options:`,
      options,
    );
    return setIntervalImmediately(() => {
      fetcher().then(
        dataPoint => {
          log.info(
            'trackers',
            `received a data point from tracker #${index}:`,
            dataPoint,
          );
        },
        error => {
          log.warn('trackers', `error in tracker #${index}:`, error);
        },
      );
    }, requestInterval * 1000);
  });

  return function stop() {
    intervalIds.forEach(intervalId => {
      clearInterval(intervalId);
    });
  };
};

function setIntervalImmediately(callback, ms, ...args) {
  callback(...args);
  return setInterval(callback, ms, args);
}
