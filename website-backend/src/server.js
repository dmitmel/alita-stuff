const http = require('http');
const EventEmitter = require('events');
const typeCheck = require('./utils/typeCheck');
const log = require('./logger');

class Server extends EventEmitter {
  constructor(config) {
    super();
    log.info('initializing server');
    typeCheck.assert(config, 'config', 'Object');
    let { hostname, port } = config;
    typeCheck.assert(hostname, 'hostname', 'String');
    typeCheck.assert(port, 'port', 'Number');

    this._hostname = hostname;
    this._port = port;
  }

  async start() {
    if (this._server != null) throw new Error('server already running');

    log.info('starting server');

    this._server = new http.Server((req, res) => this.handleRequest(req, res));

    this._server.on('error', err => log.warn('server network error:', err));

    await new Promise((resolve, _reject) => {
      this._server.listen(this._port, this._hostname, resolve);
    });

    log.info(`server is listening on http://${this._hostname}:${this._port}`);
  }

  async stop() {
    if (this._server == null) throw new Error('server aready stopped');

    log.info('stopping server');

    await new Promise((resolve, reject) => {
      this._server.close(err => {
        if (err != null) reject(err);
        else resolve();
      });
    });

    this._server = null;

    this.emit('stop');
  }

  /**
   * @param {http.IncomingMessage} req
   * @param {http.ServerResponse}  res
   */
  handleRequest(req, res) {
    log.info(
      `request from ${req.connection.remoteAddress}:${req.connection.remotePort}: ${req.url}`,
    );
    res.end('Hello world!');
    this.stop();
  }
}

module.exports = Server;
