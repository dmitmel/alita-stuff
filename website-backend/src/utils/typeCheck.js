function isBoolean(val) {
  return typeof val === 'boolean';
}

function isNumber(val) {
  return typeof val === 'number';
}

function isString(val) {
  return typeof val === 'string';
}

function isArray(val) {
  return Array.isArray(val);
}

function isObject(val) {
  return typeof val === 'object' && !isArray(val);
}

function isFunction(val) {
  return typeof val === 'function';
}

function typeOf(val) {
  if (isBoolean(val)) return 'Boolean';
  else if (isNumber(val)) return 'Number';
  else if (isString(val)) return 'String';
  else if (isArray(val)) return 'Array';
  else if (isObject(val)) return 'Object';
  else if (isFunction(val)) return 'Function';
  else return null;
}

function assert(val, valName, expectedType) {
  if (!isString(expectedType))
    throw new TypeError('expectedType must be a string');
  if (typeOf(val) !== expectedType)
    throw new TypeError(`expected type of ${valName} to be ${expectedType}`);
}

module.exports = {
  isBoolean,
  isNumber,
  isString,
  isArray,
  isObject,
  isFunction,
  typeOf,
  assert,
};
