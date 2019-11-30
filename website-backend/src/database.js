const fs = require('fs');
const byline = require('byline');
const typeCheck = require('./utils/typeCheck');
const log = require('./logger');

const DATA_FILE_MODE = 0o600; // only the owner can read and write

class Database {
  constructor(filePath) {
    typeCheck.assert(filePath, 'filePath', 'String');

    this.filePath = filePath;
    this.records = [];
  }

  init() {
    return this.read().catch(err => {
      if (err.code === 'ENOENT') {
        log.warn(`database(${this.filePath})`, err);
        log.warn(
          `database(${this.filePath})`,
          `data file doesn't exist, creating it`,
        );
        return this.write();
      } else {
        throw err;
      }
    });
  }

  read() {
    return new Promise((resolve, reject) => {
      log.info(`database(${this.filePath})`, `reading data file`);

      let stream1 = fs.createReadStream(this.filePath, {
        encoding: 'utf-8',
        mode: DATA_FILE_MODE,
      });

      let stream2 = byline.createStream(stream1);

      stream1.on('error', err => reject(err));
      stream2.on('error', err => reject(err));

      let lineNumber = 1;
      stream2.on('data', line => {
        let record;
        try {
          record = JSON.parse(line);
        } catch (err) {
          err.message = `database(${this.filePath}): failed to parse line ${lineNumber}: ${err.message}`;
          throw err;
        }
        this.records.push(record);
        lineNumber++;
      });

      stream2.on('end', () => {
        log.info(
          `database(${this.filePath})`,
          `read ${this.records.length} records`,
        );
        resolve();
      });
    });
  }

  write() {
    return new Promise((resolve, reject) => {
      log.info(`database(${this.filePath})`, 'writing into data file');

      let stream = fs.createWriteStream(this.filePath, {
        encoding: 'utf-8',
        mode: DATA_FILE_MODE,
      });

      stream.on('error', err => reject(err));
      stream.on('finish', () => resolve());

      this.records.forEach(record => {
        stream.write(`${JSON.stringify(record)}\n`);
      });
    });
  }

  push() {}
}

module.exports = Database;
