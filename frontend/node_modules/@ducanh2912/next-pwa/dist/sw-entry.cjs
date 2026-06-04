'use strict';var workboxWindow=require('workbox-window');if ("undefined" != typeof window && "serviceWorker" in navigator && "undefined" != typeof caches) {
    let e;
    if (__PWA_SW_ENTRY_WORKER__ && (e = new Worker(__PWA_SW_ENTRY_WORKER__)), window.workbox = new workboxWindow.Workbox(window.location.origin + __PWA_SW__, {
        scope: __PWA_SCOPE__
    }), __PWA_ENABLE_REGISTER__ && window.workbox.register(), __PWA_CACHE_ON_FRONT_END_NAV__ || __PWA_START_URL__) {
        let _ = async (_)=>{
            if (!window.navigator.onLine || !_) return;
            let t = _ instanceof URL ? _.toString() : "string" == typeof _ ? _ : void 0;
            if ("string" == typeof t) {
                if (__PWA_START_URL__ && t === __PWA_START_URL__) {
                    if (e) e.postMessage({
                        type: "__START_URL_CACHE__",
                        url: t
                    });
                    else {
                        let e = await fetch(_);
                        if (!e.redirected) return (await caches.open("start-url")).put(_, e);
                    }
                } else __PWA_CACHE_ON_FRONT_END_NAV__ && e?.postMessage({
                    type: "__FRONTEND_NAV_CACHE__",
                    shouldCacheAggressively: __PWA_AGGRFEN_CACHE__,
                    url: t
                });
            }
        }, t = history.pushState;
        history.pushState = (...e)=>{
            t.apply(history, e), _(e[2]);
        };
        let o = history.replaceState;
        history.replaceState = (...e)=>{
            o.apply(history, e), _(e[2]);
        }, window.addEventListener("online", ()=>{
            _(window.location.pathname);
        });
    }
    __PWA_RELOAD_ON_ONLINE__ && window.addEventListener("online", ()=>location.reload());
}