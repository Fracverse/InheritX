import { firstValueFrom } from 'rxjs';
import { horizonUrl$ } from '../state/store.mjs';

async function fetchAccountBalance(pk) {
  const horizonUrl = await firstValueFrom(horizonUrl$);
  if (!horizonUrl) {
    throw new Error("There is no Horizon URL set");
  }
  const url = new URL(horizonUrl);
  url.pathname = `/accounts/${pk}`;
  const response = await fetch(url);
  const data = await response.json();
  const nativeBalance = data.balances.find((b) => b.asset_type === "native");
  return nativeBalance.balance;
}

export { fetchAccountBalance };
//# sourceMappingURL=account.service.mjs.map
