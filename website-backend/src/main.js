const http = require('http');
const fs = require('fs');
const typeCheck = require('./utils/typeCheck');

let configPath = process.argv.length > 2 ? process.argv[2] : 'config.json';
let config = JSON.parse(fs.readFileSync(configPath).toString());

typeCheck.assert(config, 'config', 'Object');

require('./trackers')(config.trackers);

let server = http.createServer((_req, res) => {
  res.end('Hello world!');
});

server.listen(8080);
