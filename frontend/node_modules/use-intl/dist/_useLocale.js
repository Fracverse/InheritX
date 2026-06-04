'use strict';

if (process.env.NODE_ENV === 'production') {
  module.exports = require('./production/_useLocale.js');
} else {
  module.exports = require('./development/_useLocale.js');
}
