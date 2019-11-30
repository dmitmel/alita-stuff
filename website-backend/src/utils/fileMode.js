const typeCheck = require('./typeCheck');

function fileMode(options) {
  typeCheck.assert(options, 'options', 'Object');
  let { owner, group, others } = options;

  let mode = 0;
  for (let i = 0; i < 3; i++) {
    mode <<= 3;

    let permissions = [owner, group, others][i];
    if (permissions == null) continue;
    typeCheck.assert(permissions, 'permissions', 'String');

    for (let j = 0; j < permissions.length; j++) {
      let char = permissions[j];
      switch (char) {
        case 'r':
          mode |= 0b100;
          break;
        case 'w':
          mode |= 0b010;
          break;
        case 'x':
          mode |= 0b001;
          break;
      }
    }
  }

  return mode & ~process.umask();
}

module.exports = fileMode;
