const http = require('http');
const fs = require('fs');
const typeCheck = require('./utils/typeCheck');
const Database = require('./database');

let configPath = process.argv.length > 2 ? process.argv[2] : 'config.json';
let config = JSON.parse(fs.readFileSync(configPath).toString());

const startTrackers = require('./trackers');

typeCheck.assert(config, 'config', 'Object');

let db = new Database('./database2.json');
db.init().then(() => {
  db.records.forEach(r => console.log(r));
  db.records.push(new Date());
  return db.write();
});

startTrackers(config.trackers);

let server = http.createServer((_req, res) => {
  res.end('Hello world!');
});

server.listen(8080);
