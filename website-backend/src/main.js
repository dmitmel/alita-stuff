const http = require('http');
const fs = require('fs');
const typeCheck = require('./utils/typeCheck');
const log = require('./logger');
const mkdirParents = require('./utils/mkdirParents');
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

typeCheck.assert(config.database, 'config.database', 'Object');
typeCheck.assert(config.database.dir, 'config.database.dir', 'String');
let databaseDir = config.database.dir;
mkdirParents.sync(databaseDir);

async function start() {
  let stopTrackers = await startTrackers(config.trackers, databaseDir);
  shutdownCallbacks.push(() => stopTrackers());
}

start();

// let server = http.createServer((_req, res) => {
//   res.end('Hello world!');
// });

// server.listen(8080);
