'use strict';

if (process.env.NODE_ENV === 'production') {
  module.exports = require('./production/_IntlProvider.js');
} else {
  module.exports = require('./development/_IntlProvider.js');
}
