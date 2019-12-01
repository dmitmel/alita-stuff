const http = require('http');
const typeCheck = require('./utils/typeCheck');
const log = require('./logger');

class Server {
  constructor(config) {
    typeCheck.assert(config, 'config', 'Object');
    let { hostname, port } = config;
    typeCheck.assert(hostname, 'hostname', 'String');
    typeCheck.assert(port, 'port', 'Number');

    this._hostname = hostname;
    this._port = port;

    this._server = http.createServer((req, res) =>
      this.handleRequest(req, res),
    );
  }

  async start() {
    log.info('starting server');

    await new Promise((resolve, _reject) => {
      this._server.listen(this._port, this._hostname, resolve);
    });

    log.info(`server is listening on http://${this._hostname}:${this._port}`);
  }

  async stop() {
    log.info('stopping server');

    await new Promise((resolve, reject) => {
      this._server.close(err => {
        if (err != null) reject(err);
        else resolve();
      });
    });
  }

  handleRequest(req, res) {
    log.info(
      `request from ${req.connection.remoteAddress}:${req.connection.remotePort}`,
    );
    res.end('Hello world!');
  }
}

module.exports = Server;
