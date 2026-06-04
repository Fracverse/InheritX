'use strict';

var rxjs = require('rxjs');

class ReactiveState {
    constructor(host, source, value) {
        this.host = host;
        this.source = source;
        this.value = value;
        this.sub = null;
        this.value$ = new rxjs.BehaviorSubject(undefined);
        this.host.addController(this);
    }
    hostConnected() {
        this.sub = this.source.subscribe(value => {
            this.value = value;
            this.value$.next(value);
            this.host.requestUpdate();
        });
    }
    hostDisconnected() {
        this.sub?.unsubscribe();
    }
}

exports.ReactiveState = ReactiveState;
//# sourceMappingURL=reactive-state.cjs.map
