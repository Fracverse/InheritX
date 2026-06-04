import { BehaviorSubject } from 'rxjs';

class ReactiveState {
  constructor(host, source, value) {
    this.host = host;
    this.source = source;
    this.value = value;
    this.sub = null;
    this.value$ = new BehaviorSubject(void 0);
    this.host.addController(this);
  }
  hostConnected() {
    this.sub = this.source.subscribe((value) => {
      this.value = value;
      this.value$.next(value);
      this.host.requestUpdate();
    });
  }
  hostDisconnected() {
    this.sub?.unsubscribe();
  }
}

export { ReactiveState };
//# sourceMappingURL=reactive-state.mjs.map
