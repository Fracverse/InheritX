'use strict';

if (process.env.NODE_ENV === 'production') {
  module.exports = require('./production/react.js');
} else {
  module.exports = require('./development/react.js');
}
