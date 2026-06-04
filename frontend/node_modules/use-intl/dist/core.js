'use strict';

if (process.env.NODE_ENV === 'production') {
  module.exports = require('./production/core.js');
} else {
  module.exports = require('./development/core.js');
}
