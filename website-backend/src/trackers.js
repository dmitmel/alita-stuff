const fs = require('fs');
const path = require('path');
const typeCheck = require('./utils/typeCheck');

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

  let intervalIds = trackerConfigs.map(trackerConfig => {
    typeCheck.assert(trackerConfig, 'trackerConfig', 'Object');
    let { type, requestInterval, options } = trackerConfig;
    typeCheck.assert(type, 'type', 'String');
    typeCheck.assert(requestInterval, 'requestInterval', 'Number');
    typeCheck.assert(options, 'options', 'Object');

    if (!TRACKERS.has(type)) throw new Error(`unknown tracker: ${type}`);
    let createFetcher = TRACKERS.get(type);
    let fetcher = createFetcher(options);
    typeCheck.assert(fetcher, 'fetcher', 'Function');

    console.log(
      `starting tracker with type '${type}', request interval of ${requestInterval} seconds and the following options:`,
      options,
    );
    return setIntervalImmediately(() => {
      fetcher().then(
        dataPoint => {
          console.log(new Date(), dataPoint);
        },
        error => {
          console.warn(error);
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
