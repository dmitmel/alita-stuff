const fs = require('fs');
const path = require('path');
const typeCheck = require('./utils/typeCheck');
const log = require('./logger');
const PushDatabase = require('./PushDatabase');
const mkdirParents = require('./utils/mkdirParents');

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

module.exports = async function start(trackerConfigs, databaseDir) {
  typeCheck.assert(trackerConfigs, 'trackerConfigs', 'Array');
  typeCheck.assert(databaseDir, 'databaseDir', 'String');

  let intervalIds = await Promise.all(
    trackerConfigs.map(async (trackerConfig, index) => {
      typeCheck.assert(trackerConfig, 'trackerConfig', 'Object');
      let { type, id, requestInterval, options } = trackerConfig;
      typeCheck.assert(type, 'type', 'String');
      typeCheck.assert(id, 'id', 'String');
      typeCheck.assert(requestInterval, 'requestInterval', 'Number');
      typeCheck.assert(options, 'options', 'Object');

      if (!TRACKERS.has(type)) throw new Error(`unknown tracker: ${type}`);
      let createFetcher = TRACKERS.get(type);
      let fetcher = createFetcher(options);
      typeCheck.assert(fetcher, 'fetcher', 'Function');

      log.info(
        `trackers: registered tracker #${index}:\n  type: ${type}\n  id: ${id}\n  request interval: ${requestInterval} seconds\n  options:`,
        options,
      );

      log.info(`tracker(${id}): initializing database`);
      let database = new PushDatabase(path.join(databaseDir, `${id}.json`));
      await database.init();

      return setIntervalImmediately(() => {
        let timestamp = Math.floor(
          // UNIX timestamps are stored in seconds, not milliseconds
          new Date().getTime() / 1000,
        );
        fetcher().then(
          data => {
            log.info(`tracker(${id}): received a data point:`, data);
            return database.push({ timestamp, data });
          },
          error => {
            log.warn(`tracker(${id}): error:`, error);
          },
        );
      }, requestInterval * 1000);
    }),
  );

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
