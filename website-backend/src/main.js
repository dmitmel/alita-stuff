const fs = require('fs');
const path = require('path');
const EventEmitter = require('events');
const typeCheck = require('./utils/typeCheck');
const log = require('./logger');
const mkdirParents = require('./utils/mkdirParents');
const fileMode = require('./utils/fileMode');
const Trackers = require('./trackers');
const Server = require('./server');

let databaseDirMode = fileMode({ owner: 'rwx' });

class App extends EventEmitter {
  constructor(config) {
    super();
    log.info('initializing app');
    typeCheck.assert(config, 'config', 'Object');

    this.isRunning = false;
    this.isStopping = false;

    typeCheck.assert(config.database, 'config.database', 'Object');
    typeCheck.assert(config.database.dir, 'config.database.dir', 'String');
    let databaseDir = config.database.dir;
    mkdirParents.sync(databaseDir, databaseDirMode);

    let trackersDatabaseDir = path.join(databaseDir, 'trackers');
    mkdirParents.sync(trackersDatabaseDir, databaseDirMode);
    this.trackers = new Trackers(config.trackers, trackersDatabaseDir);

    this.server = new Server(config.server);

    process.on('SIGINT', signal => this.handleSignal(signal));
    // process.on('SIGTERM', signal => this.handleSignal(signal)));
  }

  async start() {
    try {
      log.info('starting app');

      await this.trackers.start();
      this.once('shutdown', () => this.trackers.stop());

      await this.server.start();
      this.server.once('stop', () => this.requestFullShutdown());
      this.once('shutdown', () => this.server.stop());
    } catch (err) {
      log.error(err);
      this.requestFullShutdown();
    }
  }

  handleSignal(signal) {
    log.info('received signal', signal);
    this.requestFullShutdown();
  }

  requestFullShutdown() {
    if (!this.isStopping) {
      this.isStopping = true;
      log.info('starting shutdown');
      this.emit('shutdown');
    }
  }
}

let configPath = process.argv.length > 2 ? process.argv[2] : 'config.json';
let config = JSON.parse(fs.readFileSync(configPath, 'utf-8'));
let app = new App(config);
app.start();
