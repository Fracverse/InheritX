'use strict';

var rxjs = require('rxjs');
var store = require('../state/store.cjs');

async function fetchAccountBalance(pk) {
    const horizonUrl = await rxjs.firstValueFrom(store.horizonUrl$);
    if (!horizonUrl) {
        throw new Error('There is no Horizon URL set');
    }
    const url = new URL(horizonUrl);
    url.pathname = `/accounts/${pk}`;
    const response = await fetch(url);
    const data = await response.json();
    const nativeBalance = data.balances.find((b) => b.asset_type === 'native');
    return nativeBalance.balance;
}

exports.fetchAccountBalance = fetchAccountBalance;
//# sourceMappingURL=account.service.cjs.map
