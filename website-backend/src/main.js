const fs = require('fs');
const path = require('path');
const typeCheck = require('./utils/typeCheck');
const log = require('./logger');
const mkdirParents = require('./utils/mkdirParents');
const fileMode = require('./utils/fileMode');
const startTrackers = require('./trackers');

let shutdownCallbacks = [];
function handleSignal(signal) {
  log.info('received signal', signal);
  log.info('starting shutdown');
  shutdownCallbacks.forEach(cb => cb());
}
process.on('SIGINT', handleSignal);
process.on('SIGTERM', handleSignal);

let configPath = process.argv.length > 2 ? process.argv[2] : 'config.json';
let config = JSON.parse(fs.readFileSync(configPath).toString());

typeCheck.assert(config, 'config', 'Object');

const DATABASE_DIR_MODE = fileMode({ owner: 'rwx' });

typeCheck.assert(config.database, 'config.database', 'Object');
typeCheck.assert(config.database.dir, 'config.database.dir', 'String');
let databaseDir = config.database.dir;
mkdirParents.sync(databaseDir, DATABASE_DIR_MODE);

(async () => {
  let trackersDatabaseDir = path.join(databaseDir, 'trackers');
  mkdirParents.sync(trackersDatabaseDir, DATABASE_DIR_MODE);
  let stopTrackers = await startTrackers(config.trackers, trackersDatabaseDir);
  shutdownCallbacks.push(() => stopTrackers());
})();

// let server = http.createServer((_req, res) => {
//   res.end('Hello world!');
// });

// server.listen(8080);
