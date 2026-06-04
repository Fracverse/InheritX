'use strict';self.fallback = async (_)=>{
    let { destination: e, url: A } = _, s = {
        document: process.env.__PWA_FALLBACK_DOCUMENT__,
        image: process.env.__PWA_FALLBACK_IMAGE__,
        audio: process.env.__PWA_FALLBACK_AUDIO__,
        video: process.env.__PWA_FALLBACK_VIDEO__,
        font: process.env.__PWA_FALLBACK_FONT__
    }[e];
    return s ? caches.match(s, {
        ignoreSearch: !0
    }) : "" === e && process.env.__PWA_FALLBACK_DATA__ && A.match(/\/_next\/data\/.+\/.+\.json$/i) ? caches.match(process.env.__PWA_FALLBACK_DATA__, {
        ignoreSearch: !0
    }) : Response.error();
};