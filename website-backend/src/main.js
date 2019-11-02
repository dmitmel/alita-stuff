const http = require('http');
const fs = require('fs');

let configPath = process.argv.length > 2 ? process.argv[2] : 'config.json';
let config = JSON.parse(fs.readFileSync(configPath).toString());

require('./trackers')(config.trackers);

let server = http.createServer((_req, res) => {
  res.end('Hello world!');
});

server.listen(8080);
