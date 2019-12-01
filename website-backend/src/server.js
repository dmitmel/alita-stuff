const http = require('http');
const typeCheck = require('./utils/typeCheck');
const log = require('./logger');

async function startServer(config) {
  typeCheck.assert(config, 'config', 'Object');
  let { host, port } = config;
  typeCheck.assert(host, 'host', 'String');
  typeCheck.assert(port, 'port', 'Number');

  log.info('starting server');

  let server = http.createServer((_req, res) => {
    res.end('Hello world!');
  });

  await new Promise((resolve, _reject) => {
    server.listen(port, host, resolve);
  });

  log.info(`server is listening on http://${host}:${port}`);

  function stop() {
    log.info('stopping server');

    return new Promise((resolve, reject) => {
      server.close(err => {
        if (err != null) reject(err);
        else resolve();
      });
    });
  }

  return stop;
}

module.exports = startServer;
