const fs = require('fs');
const path = require('path');
const typeCheck = require('./utils/typeCheck');
const log = require('./logger');
const PushDatabase = require('./PushDatabase');

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

class Trackers {
  constructor(trackerConfigs, databaseDir) {
    typeCheck.assert(trackerConfigs, 'trackerConfigs', 'Array');
    typeCheck.assert(databaseDir, 'databaseDir', 'String');

    this._databaseDir = databaseDir;
    this._trackers = trackerConfigs.map((trackerConfig, index) => {
      typeCheck.assert(trackerConfig, 'trackerConfig', 'Object');
      let { type, id, requestInterval, options } = trackerConfig;
      typeCheck.assert(type, 'type', 'String');
      typeCheck.assert(id, 'id', 'String');
      typeCheck.assert(requestInterval, 'requestInterval', 'Number');
      typeCheck.assert(options, 'options', 'Object');

      log.info(
        `trackers: initializing tracker #${index}:\n  type: ${type}\n  id: ${id}\n  request interval: ${requestInterval} seconds\n  options:`,
        options,
      );

      if (!TRACKERS.has(type)) throw new Error(`unknown tracker type: ${type}`);
      let createFetcher = TRACKERS.get(type);
      let fetcher = createFetcher(options);
      typeCheck.assert(fetcher, 'fetcher', 'Function');

      return new TrackerRunner({
        id,
        requestInterval,
        fetcher,
        databaseFile: path.join(databaseDir, `${id}.json`),
      });
    });
  }

  async start() {
    log.info('starting trackers');

    let startStatuses = await Promise.all(
      this._trackers.map(tracker => {
        return tracker.start().then(
          () => ({ status: 'success' }),
          err => {
            log.error(`tracker(${tracker.id}): error while starting:`, err);
            return { status: 'error' };
          },
        );
      }),
    );

    if (startStatuses.findIndex(({ status }) => status !== 'success') >= 0) {
      log.error(
        'failed to start some trackers (see logs above), stopping all running ones',
      );
      try {
        await this.stop();
      } catch (err) {
        log.error(err);
      }
      throw new Error('failed to start trackers');
    }
  }

  async stop() {
    log.info('stopping trackers');
    let stopStatuses = await Promise.all(
      this._trackers.map(tracker =>
        tracker.stop().then(
          () => ({ status: 'success' }),
          err => {
            log.error(`tracker(${tracker.id}): error while stopping:`, err);
            return { status: 'error' };
          },
        ),
      ),
    );

    if (stopStatuses.findIndex(({ status }) => status !== 'success') >= 0) {
      throw new Error('failed to stop some trackers (see logs above)');
    }
  }
}

class TrackerRunner {
  constructor(options) {
    typeCheck.assert(options, 'options', 'Object');
    let { id, requestInterval, fetcher, databaseFile } = options;
    typeCheck.assert(id, 'id', 'String');
    typeCheck.assert(requestInterval, 'requestInterval', 'Number');
    typeCheck.assert(fetcher, 'fetcher', 'Function');
    typeCheck.assert(databaseFile, 'databaseFile', 'String');

    this.id = id;
    this.requestInterval = requestInterval;
    this.fetcher = fetcher;
    this.databaseFile = databaseFile;
    this._intervalId = null;
    this._database = null;
  }

  async start() {
    log.info(`tracker(${this.id}): starting`);

    if (this._database == null) {
      this._database = new PushDatabase(this.databaseFile);
      await this._database.init();
    }

    if (this._intervalId == null) {
      this._intervalId = setIntervalImmediately(
        () => this._fetchDataPoint(),
        this.requestInterval * 1000,
      );
    }
  }

  async _fetchDataPoint() {
    // check if the tracker is still running
    if (this._intervalId == null || this._database == null) return;

    let timestamp = Math.floor(
      // UNIX timestamps are stored in seconds, not milliseconds
      new Date().getTime() / 1000,
    );
    let data;
    try {
      data = await this.fetcher();
    } catch (err) {
      log.warn(`tracker(${this.id}): error:`, err);
      return;
    }

    log.info(`tracker(${this.id}): received a data point:`, data);
    await this._database.push({ timestamp, data });
  }

  async stop() {
    log.info(`tracker(${this.id}): stopping`);

    if (this._intervalId != null) {
      clearInterval(this._intervalId);
      this._intervalId = null;
    }

    if (this._database != null) {
      await this._database.write();
      this._database = null;
    }
  }
}

function setIntervalImmediately(callback, ms, ...args) {
  callback(...args);
  return setInterval(callback, ms, args);
}

module.exports = Trackers;
