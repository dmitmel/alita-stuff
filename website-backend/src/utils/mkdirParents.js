// see: https://github.com/substack/node-mkdirp/blob/master/index.js
// and: https://doc.rust-lang.org/src/std/fs.rs.html#2108-2128

const fs = require('fs');
const path = require('path');

function mkdirParentsSync(p, mode) {
  p = path.resolve(p);

  try {
    fs.mkdirSync(p, mode);
  } catch (err) {
    if (err.code === 'ENOENT') {
      mkdirParentsSync(path.dirname(p), mode);
      mkdirParentsSync(p, mode);
    } else if (!isDirectorySync(p)) {
      throw err;
    }
  }
}

function isDirectorySync(p) {
  let stat;
  try {
    stat = fs.statSync(p);
  } catch (_err) {
    return false;
  }
  return stat.isDirectory();
}

module.exports = { sync: mkdirParentsSync };
