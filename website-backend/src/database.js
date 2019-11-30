const fs = require('fs');
const byline = require('byline');
const typeCheck = require('./utils/typeCheck');
const log = require('./logger');

const COMMON_DATA_FILE_OPTIONS = {
  encoding: 'utf-8',
  mode: 0o600, // only the owner can read and write
};

class Database {
  constructor(filePath) {
    typeCheck.assert(filePath, 'filePath', 'String');

    this.filePath = filePath;
    this.records = [];
  }

  init() {
    return this.read().catch(err => {
      if (err.code === 'ENOENT') {
        log.warn(`database(${this.filePath}):`, err);
        log.warn(
          `database(${this.filePath}): data file doesn't exist, creating it`,
        );
        return this.write();
      } else {
        throw err;
      }
    });
  }

  read() {
    return new Promise((resolve, reject) => {
      log.info(`database(${this.filePath}): reading data file`);

      let stream1 = fs.createReadStream(
        this.filePath,
        COMMON_DATA_FILE_OPTIONS,
      );

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
          `database(${this.filePath}): read ${this.records.length} records`,
        );
        resolve();
      });
    });
  }

  write() {
    return new Promise((resolve, reject) => {
      log.info(`database(${this.filePath}): writing into data file`);

      let stream = fs.createWriteStream(
        this.filePath,
        COMMON_DATA_FILE_OPTIONS,
      );

      stream.on('error', err => reject(err));
      stream.on('finish', () => resolve());

      this.records.forEach(record => {
        stream.write(`${JSON.stringify(record)}\n`);
      });
    });
  }

  push(record) {
    return new Promise((resolve, reject) => {
      this.records.push(record);
      fs.appendFile(
        this.filePath,
        `${JSON.stringify(record)}\n`,
        COMMON_DATA_FILE_OPTIONS,
        err => {
          if (err != null) reject(err);
          else resolve();
        },
      );
    });
  }
}

module.exports = Database;
