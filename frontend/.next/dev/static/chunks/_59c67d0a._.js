(globalThis.TURBOPACK || (globalThis.TURBOPACK = [])).push([typeof document === "object" ? document.currentScript : undefined,
"[project]/context/WalletContext.tsx [app-client] (ecmascript)", ((__turbopack_context__) => {
"use strict";

__turbopack_context__.s([
    "WalletProvider",
    ()=>WalletProvider,
    "useWallet",
    ()=>useWallet
]);
var __TURBOPACK__imported__module__$5b$project$5d2f$node_modules$2f$next$2f$dist$2f$compiled$2f$react$2f$jsx$2d$dev$2d$runtime$2e$js__$5b$app$2d$client$5d$__$28$ecmascript$29$__ = __turbopack_context__.i("[project]/node_modules/next/dist/compiled/react/jsx-dev-runtime.js [app-client] (ecmascript)");
var __TURBOPACK__imported__module__$5b$project$5d2f$node_modules$2f$next$2f$dist$2f$compiled$2f$react$2f$index$2e$js__$5b$app$2d$client$5d$__$28$ecmascript$29$__ = __turbopack_context__.i("[project]/node_modules/next/dist/compiled/react/index.js [app-client] (ecmascript)");
var __TURBOPACK__imported__module__$5b$project$5d2f$node_modules$2f40$creit$2e$tech$2f$stellar$2d$wallets$2d$kit$2f$index$2e$mjs__$5b$app$2d$client$5d$__$28$ecmascript$29$__$3c$locals$3e$__ = __turbopack_context__.i("[project]/node_modules/@creit.tech/stellar-wallets-kit/index.mjs [app-client] (ecmascript) <locals>");
var __TURBOPACK__imported__module__$5b$project$5d2f$node_modules$2f40$creit$2e$tech$2f$stellar$2d$wallets$2d$kit$2f$stellar$2d$wallets$2d$kit$2e$mjs__$5b$app$2d$client$5d$__$28$ecmascript$29$__ = __turbopack_context__.i("[project]/node_modules/@creit.tech/stellar-wallets-kit/stellar-wallets-kit.mjs [app-client] (ecmascript)");
var __TURBOPACK__imported__module__$5b$project$5d2f$node_modules$2f40$creit$2e$tech$2f$stellar$2d$wallets$2d$kit$2f$types$2e$mjs__$5b$app$2d$client$5d$__$28$ecmascript$29$__ = __turbopack_context__.i("[project]/node_modules/@creit.tech/stellar-wallets-kit/types.mjs [app-client] (ecmascript)");
var __TURBOPACK__imported__module__$5b$project$5d2f$node_modules$2f40$creit$2e$tech$2f$stellar$2d$wallets$2d$kit$2f$utils$2e$mjs__$5b$app$2d$client$5d$__$28$ecmascript$29$__ = __turbopack_context__.i("[project]/node_modules/@creit.tech/stellar-wallets-kit/utils.mjs [app-client] (ecmascript)");
var __TURBOPACK__imported__module__$5b$project$5d2f$node_modules$2f$next$2f$navigation$2e$js__$5b$app$2d$client$5d$__$28$ecmascript$29$__ = __turbopack_context__.i("[project]/node_modules/next/navigation.js [app-client] (ecmascript)");
;
var _s = __turbopack_context__.k.signature(), _s1 = __turbopack_context__.k.signature();
"use client";
;
;
;
const WalletContext = /*#__PURE__*/ (0, __TURBOPACK__imported__module__$5b$project$5d2f$node_modules$2f$next$2f$dist$2f$compiled$2f$react$2f$index$2e$js__$5b$app$2d$client$5d$__$28$ecmascript$29$__["createContext"])(undefined);
const useWallet = ()=>{
    _s();
    const context = (0, __TURBOPACK__imported__module__$5b$project$5d2f$node_modules$2f$next$2f$dist$2f$compiled$2f$react$2f$index$2e$js__$5b$app$2d$client$5d$__$28$ecmascript$29$__["useContext"])(WalletContext);
    if (!context) {
        throw new Error("useWallet must be used within a WalletProvider");
    }
    return context;
};
_s(useWallet, "b9L3QQ+jgeyIrH0NfHrJ8nn7VMU=");
const WalletProvider = ({ children })=>{
    _s1();
    const [address, setAddress] = (0, __TURBOPACK__imported__module__$5b$project$5d2f$node_modules$2f$next$2f$dist$2f$compiled$2f$react$2f$index$2e$js__$5b$app$2d$client$5d$__$28$ecmascript$29$__["useState"])(null);
    const [isConnecting, setIsConnecting] = (0, __TURBOPACK__imported__module__$5b$project$5d2f$node_modules$2f$next$2f$dist$2f$compiled$2f$react$2f$index$2e$js__$5b$app$2d$client$5d$__$28$ecmascript$29$__["useState"])(false);
    const [selectedWalletId, setSelectedWalletId] = (0, __TURBOPACK__imported__module__$5b$project$5d2f$node_modules$2f$next$2f$dist$2f$compiled$2f$react$2f$index$2e$js__$5b$app$2d$client$5d$__$28$ecmascript$29$__["useState"])(null);
    const [kit, setKit] = (0, __TURBOPACK__imported__module__$5b$project$5d2f$node_modules$2f$next$2f$dist$2f$compiled$2f$react$2f$index$2e$js__$5b$app$2d$client$5d$__$28$ecmascript$29$__["useState"])(null);
    const [isModalOpen, setIsModalOpen] = (0, __TURBOPACK__imported__module__$5b$project$5d2f$node_modules$2f$next$2f$dist$2f$compiled$2f$react$2f$index$2e$js__$5b$app$2d$client$5d$__$28$ecmascript$29$__["useState"])(false);
    const router = (0, __TURBOPACK__imported__module__$5b$project$5d2f$node_modules$2f$next$2f$navigation$2e$js__$5b$app$2d$client$5d$__$28$ecmascript$29$__["useRouter"])();
    // Initialize kit on mount
    (0, __TURBOPACK__imported__module__$5b$project$5d2f$node_modules$2f$next$2f$dist$2f$compiled$2f$react$2f$index$2e$js__$5b$app$2d$client$5d$__$28$ecmascript$29$__["useEffect"])({
        "WalletProvider.useEffect": ()=>{
            const walletKit = new __TURBOPACK__imported__module__$5b$project$5d2f$node_modules$2f40$creit$2e$tech$2f$stellar$2d$wallets$2d$kit$2f$stellar$2d$wallets$2d$kit$2e$mjs__$5b$app$2d$client$5d$__$28$ecmascript$29$__["StellarWalletsKit"]({
                network: __TURBOPACK__imported__module__$5b$project$5d2f$node_modules$2f40$creit$2e$tech$2f$stellar$2d$wallets$2d$kit$2f$types$2e$mjs__$5b$app$2d$client$5d$__$28$ecmascript$29$__["WalletNetwork"].TESTNET,
                selectedWalletId: "freighter",
                modules: (0, __TURBOPACK__imported__module__$5b$project$5d2f$node_modules$2f40$creit$2e$tech$2f$stellar$2d$wallets$2d$kit$2f$utils$2e$mjs__$5b$app$2d$client$5d$__$28$ecmascript$29$__["allowAllModules"])()
            });
            setKit(walletKit);
            // Check for persisted session
            const checkSession = {
                "WalletProvider.useEffect.checkSession": async ()=>{
                    // Basic persistence check - in a real app might verify token/session
                    // For now, we rely on the kit's internal state if it has one, or we can check simple localStorage
                    // But purely client-side:
                    const savedAddress = localStorage.getItem("inheritx_wallet_address");
                    const savedWalletId = localStorage.getItem("inheritx_wallet_id");
                    if (savedAddress && savedWalletId) {
                        setAddress(savedAddress);
                        setSelectedWalletId(savedWalletId);
                    // We technically aren't "connected" in the kit sense until we call something,
                    // but for UI purposes we show the address.
                    // A robust implementation would verify connection here.
                    }
                }
            }["WalletProvider.useEffect.checkSession"];
            checkSession();
        }
    }["WalletProvider.useEffect"], []);
    const supportedWallets = [
        {
            id: "freighter",
            name: "Freighter",
            icon: "/icons/freighter.png"
        },
        {
            id: "albedo",
            name: "Albedo",
            icon: "/icons/albedo.png"
        },
        {
            id: "xbull",
            name: "xBull",
            icon: "/icons/xbull.png"
        },
        {
            id: "rabet",
            name: "Rabet",
            icon: "/icons/rabet.png"
        },
        {
            id: "lobstr",
            name: "Lobstr",
            icon: "/icons/rabet.png"
        }
    ];
    const connectCustom = async (moduleId)=>{
        if (!kit) return;
        setIsConnecting(true);
        try {
            // Set the wallet module
            kit.setWallet(moduleId);
            // Request address (triggers popup)
            const { address } = await kit.getAddress();
            setAddress(address);
            setSelectedWalletId(moduleId);
            localStorage.setItem("inheritx_wallet_address", address);
            localStorage.setItem("inheritx_wallet_id", moduleId);
            setIsModalOpen(false);
            router.push("/asset-owner");
        } catch (error) {
            console.error("Connection failed:", error);
            // Handle specific errors (user rejected, extension not found)
            throw error;
        } finally{
            setIsConnecting(false);
        }
    };
    const disconnect = async ()=>{
        setAddress(null);
        setSelectedWalletId(null);
        localStorage.removeItem("inheritx_wallet_address");
        localStorage.removeItem("inheritx_wallet_id");
    // kit.disconnect() if available
    };
    const openModal = ()=>setIsModalOpen(true);
    const closeModal = ()=>setIsModalOpen(false);
    return /*#__PURE__*/ (0, __TURBOPACK__imported__module__$5b$project$5d2f$node_modules$2f$next$2f$dist$2f$compiled$2f$react$2f$jsx$2d$dev$2d$runtime$2e$js__$5b$app$2d$client$5d$__$28$ecmascript$29$__["jsxDEV"])(WalletContext.Provider, {
        value: {
            connect: connectCustom,
            disconnect,
            address,
            isConnected: !!address,
            isConnecting,
            selectedWalletId,
            kit,
            openModal,
            closeModal,
            isModalOpen,
            supportedWallets
        },
        children: children
    }, void 0, false, {
        fileName: "[project]/context/WalletContext.tsx",
        lineNumber: 118,
        columnNumber: 5
    }, ("TURBOPACK compile-time value", void 0));
};
_s1(WalletProvider, "Ihu/ohDW936/tHiEbZrEjBIqY8g=", false, function() {
    return [
        __TURBOPACK__imported__module__$5b$project$5d2f$node_modules$2f$next$2f$navigation$2e$js__$5b$app$2d$client$5d$__$28$ecmascript$29$__["useRouter"]
    ];
});
_c = WalletProvider;
var _c;
__turbopack_context__.k.register(_c, "WalletProvider");
if (typeof globalThis.$RefreshHelpers$ === 'object' && globalThis.$RefreshHelpers !== null) {
    __turbopack_context__.k.registerExports(__turbopack_context__.m, globalThis.$RefreshHelpers$);
}
}),
"[project]/components/userIcon.tsx [app-client] (ecmascript)", ((__turbopack_context__) => {
"use strict";

__turbopack_context__.s([
    "default",
    ()=>UserIcon
]);
var __TURBOPACK__imported__module__$5b$project$5d2f$node_modules$2f$next$2f$dist$2f$compiled$2f$react$2f$jsx$2d$dev$2d$runtime$2e$js__$5b$app$2d$client$5d$__$28$ecmascript$29$__ = __turbopack_context__.i("[project]/node_modules/next/dist/compiled/react/jsx-dev-runtime.js [app-client] (ecmascript)");
;
function UserIcon() {
    return /*#__PURE__*/ (0, __TURBOPACK__imported__module__$5b$project$5d2f$node_modules$2f$next$2f$dist$2f$compiled$2f$react$2f$jsx$2d$dev$2d$runtime$2e$js__$5b$app$2d$client$5d$__$28$ecmascript$29$__["jsxDEV"])("svg", {
        width: "20",
        height: "20",
        viewBox: "0 0 16 16",
        fill: "none",
        xmlns: "http://www.w3.org/2000/svg",
        children: [
            /*#__PURE__*/ (0, __TURBOPACK__imported__module__$5b$project$5d2f$node_modules$2f$next$2f$dist$2f$compiled$2f$react$2f$jsx$2d$dev$2d$runtime$2e$js__$5b$app$2d$client$5d$__$28$ecmascript$29$__["jsxDEV"])("g", {
                clipPath: "url(#clip0_1437_27115)",
                children: /*#__PURE__*/ (0, __TURBOPACK__imported__module__$5b$project$5d2f$node_modules$2f$next$2f$dist$2f$compiled$2f$react$2f$jsx$2d$dev$2d$runtime$2e$js__$5b$app$2d$client$5d$__$28$ecmascript$29$__["jsxDEV"])("path", {
                    d: "M13.84 6.50688C13.7092 6.52877 13.5925 6.60171 13.5155 6.70966C13.4385 6.81761 13.4075 6.95172 13.4294 7.08251C13.4802 7.3857 13.5057 7.69259 13.5056 8.00001C13.5068 9.34638 13.012 10.646 12.1156 11.6506C11.558 10.8426 10.7739 10.2174 9.86188 9.85376C10.3518 9.46791 10.7092 8.939 10.8846 8.34057C11.0599 7.74214 11.0444 7.10394 10.8403 6.51472C10.6361 5.92551 10.2534 5.41456 9.74534 5.05294C9.23731 4.69132 8.62922 4.49699 8.00563 4.49699C7.38204 4.49699 6.77395 4.69132 6.26592 5.05294C5.75789 5.41456 5.37518 5.92551 5.171 6.51472C4.96682 7.10394 4.95133 7.74214 5.12668 8.34057C5.30202 8.939 5.6595 9.46791 6.14938 9.85376C5.23739 10.2174 4.45328 10.8426 3.89563 11.6506C3.19233 10.8581 2.73293 9.8793 2.57266 8.83191C2.41239 7.78451 2.55808 6.71312 2.99222 5.74655C3.42635 4.77998 4.13044 3.95939 5.01983 3.38345C5.90922 2.80751 6.94605 2.50073 8.00563 2.50001C8.31305 2.49996 8.61994 2.52546 8.92313 2.57626C9.05333 2.5968 9.18637 2.5651 9.29331 2.48806C9.40026 2.41102 9.47246 2.29487 9.49421 2.16487C9.51596 2.03487 9.48549 1.90154 9.40945 1.79389C9.3334 1.68623 9.21792 1.61296 9.08813 1.59001C7.73028 1.36158 6.33497 1.57059 5.10365 2.18687C3.87234 2.80314 2.86873 3.79479 2.23775 5.01863C1.60676 6.24247 1.38104 7.63518 1.59318 8.99567C1.80532 10.3562 2.44434 11.614 3.41797 12.5877C4.39161 13.5613 5.64948 14.2003 7.00997 14.4125C8.37046 14.6246 9.76317 14.3989 10.987 13.7679C12.2109 13.1369 13.2025 12.1333 13.8188 10.902C14.435 9.67067 14.6441 8.27536 14.4156 6.91751C14.3937 6.78672 14.3208 6.66999 14.2129 6.59299C14.1049 6.51598 13.9708 6.48501 13.84 6.50688ZM6.00563 7.50001C6.00563 7.10444 6.12293 6.71776 6.34269 6.38887C6.56246 6.05997 6.87481 5.80362 7.24026 5.65225C7.60572 5.50087 8.00785 5.46127 8.39581 5.53844C8.78377 5.61561 9.14014 5.80609 9.41985 6.08579C9.69955 6.3655 9.89003 6.72186 9.9672 7.10983C10.0444 7.49779 10.0048 7.89992 9.85339 8.26537C9.70202 8.63083 9.44567 8.94318 9.11677 9.16295C8.78787 9.38271 8.40119 9.50001 8.00563 9.50001C7.4752 9.50001 6.96649 9.28929 6.59142 8.91422C6.21635 8.53915 6.00563 8.03044 6.00563 7.50001ZM4.63563 12.3438C4.99734 11.7781 5.49564 11.3125 6.08458 10.99C6.67353 10.6676 7.33418 10.4985 8.00563 10.4985C8.67708 10.4985 9.33773 10.6676 9.92668 10.99C10.5156 11.3125 11.0139 11.7781 11.3756 12.3438C10.4121 13.0931 9.22627 13.5 8.00563 13.5C6.78499 13.5 5.59917 13.0931 4.63563 12.3438ZM14.8594 2.85376L12.8594 4.85376C12.8129 4.90025 12.7578 4.93712 12.6971 4.96229C12.6364 4.98745 12.5713 5.0004 12.5056 5.0004C12.4399 5.0004 12.3749 4.98745 12.3142 4.96229C12.2535 4.93712 12.1983 4.90025 12.1519 4.85376L11.1519 3.85376C11.1054 3.8073 11.0686 3.75215 11.0434 3.69146C11.0183 3.63076 11.0054 3.5657 11.0054 3.50001C11.0054 3.43431 11.0183 3.36926 11.0434 3.30856C11.0686 3.24786 11.1054 3.19271 11.1519 3.14626C11.2457 3.05244 11.3729 2.99973 11.5056 2.99973C11.5713 2.99973 11.6364 3.01267 11.6971 3.03781C11.7578 3.06295 11.8129 3.0998 11.8594 3.14626L12.5056 3.79313L14.1519 2.14626C14.1983 2.0998 14.2535 2.06295 14.3142 2.03781C14.3749 2.01267 14.4399 1.99973 14.5056 1.99973C14.5713 1.99973 14.6364 2.01267 14.6971 2.03781C14.7578 2.06295 14.8129 2.0998 14.8594 2.14626C14.9058 2.19271 14.9427 2.24786 14.9678 2.30856C14.993 2.36926 15.0059 2.43431 15.0059 2.50001C15.0059 2.5657 14.993 2.63076 14.9678 2.69146C14.9427 2.75215 14.9058 2.8073 14.8594 2.85376Z",
                    fill: "#FCFFFF"
                }, void 0, false, {
                    fileName: "[project]/components/userIcon.tsx",
                    lineNumber: 11,
                    columnNumber: 9
                }, this)
            }, void 0, false, {
                fileName: "[project]/components/userIcon.tsx",
                lineNumber: 10,
                columnNumber: 7
            }, this),
            /*#__PURE__*/ (0, __TURBOPACK__imported__module__$5b$project$5d2f$node_modules$2f$next$2f$dist$2f$compiled$2f$react$2f$jsx$2d$dev$2d$runtime$2e$js__$5b$app$2d$client$5d$__$28$ecmascript$29$__["jsxDEV"])("defs", {
                children: /*#__PURE__*/ (0, __TURBOPACK__imported__module__$5b$project$5d2f$node_modules$2f$next$2f$dist$2f$compiled$2f$react$2f$jsx$2d$dev$2d$runtime$2e$js__$5b$app$2d$client$5d$__$28$ecmascript$29$__["jsxDEV"])("clipPath", {
                    id: "clip0_1437_27115",
                    children: /*#__PURE__*/ (0, __TURBOPACK__imported__module__$5b$project$5d2f$node_modules$2f$next$2f$dist$2f$compiled$2f$react$2f$jsx$2d$dev$2d$runtime$2e$js__$5b$app$2d$client$5d$__$28$ecmascript$29$__["jsxDEV"])("rect", {
                        width: "16",
                        height: "16",
                        fill: "white"
                    }, void 0, false, {
                        fileName: "[project]/components/userIcon.tsx",
                        lineNumber: 18,
                        columnNumber: 11
                    }, this)
                }, void 0, false, {
                    fileName: "[project]/components/userIcon.tsx",
                    lineNumber: 17,
                    columnNumber: 9
                }, this)
            }, void 0, false, {
                fileName: "[project]/components/userIcon.tsx",
                lineNumber: 16,
                columnNumber: 7
            }, this)
        ]
    }, void 0, true, {
        fileName: "[project]/components/userIcon.tsx",
        lineNumber: 3,
        columnNumber: 5
    }, this);
}
_c = UserIcon;
var _c;
__turbopack_context__.k.register(_c, "UserIcon");
if (typeof globalThis.$RefreshHelpers$ === 'object' && globalThis.$RefreshHelpers !== null) {
    __turbopack_context__.k.registerExports(__turbopack_context__.m, globalThis.$RefreshHelpers$);
}
}),
"[project]/components/WalletModal.tsx [app-client] (ecmascript)", ((__turbopack_context__) => {
"use strict";

__turbopack_context__.s([
    "WalletModal",
    ()=>WalletModal
]);
var __TURBOPACK__imported__module__$5b$project$5d2f$node_modules$2f$next$2f$dist$2f$compiled$2f$react$2f$jsx$2d$dev$2d$runtime$2e$js__$5b$app$2d$client$5d$__$28$ecmascript$29$__ = __turbopack_context__.i("[project]/node_modules/next/dist/compiled/react/jsx-dev-runtime.js [app-client] (ecmascript)");
var __TURBOPACK__imported__module__$5b$project$5d2f$context$2f$WalletContext$2e$tsx__$5b$app$2d$client$5d$__$28$ecmascript$29$__ = __turbopack_context__.i("[project]/context/WalletContext.tsx [app-client] (ecmascript)");
var __TURBOPACK__imported__module__$5b$project$5d2f$node_modules$2f$framer$2d$motion$2f$dist$2f$es$2f$render$2f$components$2f$motion$2f$proxy$2e$mjs__$5b$app$2d$client$5d$__$28$ecmascript$29$__ = __turbopack_context__.i("[project]/node_modules/framer-motion/dist/es/render/components/motion/proxy.mjs [app-client] (ecmascript)");
var __TURBOPACK__imported__module__$5b$project$5d2f$node_modules$2f$framer$2d$motion$2f$dist$2f$es$2f$components$2f$AnimatePresence$2f$index$2e$mjs__$5b$app$2d$client$5d$__$28$ecmascript$29$__ = __turbopack_context__.i("[project]/node_modules/framer-motion/dist/es/components/AnimatePresence/index.mjs [app-client] (ecmascript)");
var __TURBOPACK__imported__module__$5b$project$5d2f$node_modules$2f$lucide$2d$react$2f$dist$2f$esm$2f$icons$2f$wallet$2e$js__$5b$app$2d$client$5d$__$28$ecmascript$29$__$3c$export__default__as__Wallet$3e$__ = __turbopack_context__.i("[project]/node_modules/lucide-react/dist/esm/icons/wallet.js [app-client] (ecmascript) <export default as Wallet>");
var __TURBOPACK__imported__module__$5b$project$5d2f$node_modules$2f$lucide$2d$react$2f$dist$2f$esm$2f$icons$2f$check$2e$js__$5b$app$2d$client$5d$__$28$ecmascript$29$__$3c$export__default__as__Check$3e$__ = __turbopack_context__.i("[project]/node_modules/lucide-react/dist/esm/icons/check.js [app-client] (ecmascript) <export default as Check>");
var __TURBOPACK__imported__module__$5b$project$5d2f$node_modules$2f$next$2f$dist$2f$compiled$2f$react$2f$index$2e$js__$5b$app$2d$client$5d$__$28$ecmascript$29$__ = __turbopack_context__.i("[project]/node_modules/next/dist/compiled/react/index.js [app-client] (ecmascript)");
var __TURBOPACK__imported__module__$5b$project$5d2f$components$2f$userIcon$2e$tsx__$5b$app$2d$client$5d$__$28$ecmascript$29$__ = __turbopack_context__.i("[project]/components/userIcon.tsx [app-client] (ecmascript)");
;
var _s = __turbopack_context__.k.signature();
"use client";
;
;
;
;
;
function WalletModal() {
    _s();
    const { isModalOpen, closeModal, supportedWallets, connect, isConnecting } = (0, __TURBOPACK__imported__module__$5b$project$5d2f$context$2f$WalletContext$2e$tsx__$5b$app$2d$client$5d$__$28$ecmascript$29$__["useWallet"])();
    // If we wanted to "select" first before connecting, we'd need local state.
    // But standard flow is click -> connect.
    // However, the screenshot shows "Connect Wallet" button at the bottom.
    // This implies: Select a wallet (radio) -> Click specific Connect button.
    // I will implement that flow.
    const [activeSelection, setActiveSelection] = __TURBOPACK__imported__module__$5b$project$5d2f$node_modules$2f$next$2f$dist$2f$compiled$2f$react$2f$index$2e$js__$5b$app$2d$client$5d$__$28$ecmascript$29$__["default"].useState(null);
    // Reset selection when modal opens
    __TURBOPACK__imported__module__$5b$project$5d2f$node_modules$2f$next$2f$dist$2f$compiled$2f$react$2f$index$2e$js__$5b$app$2d$client$5d$__$28$ecmascript$29$__["default"].useEffect({
        "WalletModal.useEffect": ()=>{
            if (isModalOpen) setActiveSelection(null);
        }
    }["WalletModal.useEffect"], [
        isModalOpen
    ]);
    const handleConnectClick = ()=>{
        if (activeSelection) {
            connect(activeSelection);
        }
    };
    return /*#__PURE__*/ (0, __TURBOPACK__imported__module__$5b$project$5d2f$node_modules$2f$next$2f$dist$2f$compiled$2f$react$2f$jsx$2d$dev$2d$runtime$2e$js__$5b$app$2d$client$5d$__$28$ecmascript$29$__["jsxDEV"])(__TURBOPACK__imported__module__$5b$project$5d2f$node_modules$2f$framer$2d$motion$2f$dist$2f$es$2f$components$2f$AnimatePresence$2f$index$2e$mjs__$5b$app$2d$client$5d$__$28$ecmascript$29$__["AnimatePresence"], {
        children: isModalOpen && /*#__PURE__*/ (0, __TURBOPACK__imported__module__$5b$project$5d2f$node_modules$2f$next$2f$dist$2f$compiled$2f$react$2f$jsx$2d$dev$2d$runtime$2e$js__$5b$app$2d$client$5d$__$28$ecmascript$29$__["jsxDEV"])(__TURBOPACK__imported__module__$5b$project$5d2f$node_modules$2f$next$2f$dist$2f$compiled$2f$react$2f$jsx$2d$dev$2d$runtime$2e$js__$5b$app$2d$client$5d$__$28$ecmascript$29$__["Fragment"], {
            children: [
                /*#__PURE__*/ (0, __TURBOPACK__imported__module__$5b$project$5d2f$node_modules$2f$next$2f$dist$2f$compiled$2f$react$2f$jsx$2d$dev$2d$runtime$2e$js__$5b$app$2d$client$5d$__$28$ecmascript$29$__["jsxDEV"])("div", {
                    className: "fixed inset-0 z-40 bg-transparent",
                    onClick: closeModal
                }, void 0, false, {
                    fileName: "[project]/components/WalletModal.tsx",
                    lineNumber: 39,
                    columnNumber: 11
                }, this),
                /*#__PURE__*/ (0, __TURBOPACK__imported__module__$5b$project$5d2f$node_modules$2f$next$2f$dist$2f$compiled$2f$react$2f$jsx$2d$dev$2d$runtime$2e$js__$5b$app$2d$client$5d$__$28$ecmascript$29$__["jsxDEV"])("div", {
                    className: " bg-[#161E22CC]",
                    children: /*#__PURE__*/ (0, __TURBOPACK__imported__module__$5b$project$5d2f$node_modules$2f$next$2f$dist$2f$compiled$2f$react$2f$jsx$2d$dev$2d$runtime$2e$js__$5b$app$2d$client$5d$__$28$ecmascript$29$__["jsxDEV"])(__TURBOPACK__imported__module__$5b$project$5d2f$node_modules$2f$framer$2d$motion$2f$dist$2f$es$2f$render$2f$components$2f$motion$2f$proxy$2e$mjs__$5b$app$2d$client$5d$__$28$ecmascript$29$__["motion"].div, {
                        initial: {
                            opacity: 0,
                            y: -10,
                            x: 20
                        },
                        animate: {
                            opacity: 1,
                            y: 0,
                            x: 0
                        },
                        exit: {
                            opacity: 0,
                            y: -10,
                            x: 20
                        },
                        transition: {
                            duration: 0.2,
                            ease: "easeOut"
                        },
                        className: "fixed top-24 right-6 z-50 w-120  rounded-4xl bg-[#161E22CC] p-8 shadow-2xl",
                        children: /*#__PURE__*/ (0, __TURBOPACK__imported__module__$5b$project$5d2f$node_modules$2f$next$2f$dist$2f$compiled$2f$react$2f$jsx$2d$dev$2d$runtime$2e$js__$5b$app$2d$client$5d$__$28$ecmascript$29$__["jsxDEV"])("div", {
                            className: "border border-[#2A3338] bg-[#161E22] rounded-4xl p-[32px] flex flex-col items-center",
                            children: [
                                /*#__PURE__*/ (0, __TURBOPACK__imported__module__$5b$project$5d2f$node_modules$2f$next$2f$dist$2f$compiled$2f$react$2f$jsx$2d$dev$2d$runtime$2e$js__$5b$app$2d$client$5d$__$28$ecmascript$29$__["jsxDEV"])("div", {
                                    className: "text-center mb-2",
                                    children: [
                                        /*#__PURE__*/ (0, __TURBOPACK__imported__module__$5b$project$5d2f$node_modules$2f$next$2f$dist$2f$compiled$2f$react$2f$jsx$2d$dev$2d$runtime$2e$js__$5b$app$2d$client$5d$__$28$ecmascript$29$__["jsxDEV"])("h2", {
                                            className: "text-2xl font-medium text-white",
                                            children: "Connect Wallet"
                                        }, void 0, false, {
                                            fileName: "[project]/components/WalletModal.tsx",
                                            lineNumber: 53,
                                            columnNumber: 19
                                        }, this),
                                        /*#__PURE__*/ (0, __TURBOPACK__imported__module__$5b$project$5d2f$node_modules$2f$next$2f$dist$2f$compiled$2f$react$2f$jsx$2d$dev$2d$runtime$2e$js__$5b$app$2d$client$5d$__$28$ecmascript$29$__["jsxDEV"])("p", {
                                            className: "mt-2 text-[#92A5A8] text-sm",
                                            children: "Connect your wallet to get started with InheritX"
                                        }, void 0, false, {
                                            fileName: "[project]/components/WalletModal.tsx",
                                            lineNumber: 56,
                                            columnNumber: 19
                                        }, this)
                                    ]
                                }, void 0, true, {
                                    fileName: "[project]/components/WalletModal.tsx",
                                    lineNumber: 52,
                                    columnNumber: 17
                                }, this),
                                /*#__PURE__*/ (0, __TURBOPACK__imported__module__$5b$project$5d2f$node_modules$2f$next$2f$dist$2f$compiled$2f$react$2f$jsx$2d$dev$2d$runtime$2e$js__$5b$app$2d$client$5d$__$28$ecmascript$29$__["jsxDEV"])("div", {
                                    className: "flex flex-col gap-3 mt-8 mb-8",
                                    children: supportedWallets.map((wallet)=>{
                                        const isSelected = activeSelection === wallet.id;
                                        return /*#__PURE__*/ (0, __TURBOPACK__imported__module__$5b$project$5d2f$node_modules$2f$next$2f$dist$2f$compiled$2f$react$2f$jsx$2d$dev$2d$runtime$2e$js__$5b$app$2d$client$5d$__$28$ecmascript$29$__["jsxDEV"])("div", {
                                            className: "group flex items-center gap-4",
                                            children: [
                                                /*#__PURE__*/ (0, __TURBOPACK__imported__module__$5b$project$5d2f$node_modules$2f$next$2f$dist$2f$compiled$2f$react$2f$jsx$2d$dev$2d$runtime$2e$js__$5b$app$2d$client$5d$__$28$ecmascript$29$__["jsxDEV"])("div", {
                                                    className: "w-1.5 h-8 group-hover:bg-[#1C252A] flex items-center justify-center transition-colors"
                                                }, void 0, false, {
                                                    fileName: "[project]/components/WalletModal.tsx",
                                                    lineNumber: 66,
                                                    columnNumber: 25
                                                }, this),
                                                /*#__PURE__*/ (0, __TURBOPACK__imported__module__$5b$project$5d2f$node_modules$2f$next$2f$dist$2f$compiled$2f$react$2f$jsx$2d$dev$2d$runtime$2e$js__$5b$app$2d$client$5d$__$28$ecmascript$29$__["jsxDEV"])("button", {
                                                    onClick: ()=>setActiveSelection(wallet.id),
                                                    className: `flex group-hover:bg-[#1C252A] items-center gap-4 w-full p-4 rounded-e-2xl transition-all border ${isSelected ? "bg-[#1a2333] border-[#33C5E0]/30" : "bg-transparent border-transparent hover:bg-[#1a2333]"}`,
                                                    children: [
                                                        /*#__PURE__*/ (0, __TURBOPACK__imported__module__$5b$project$5d2f$node_modules$2f$next$2f$dist$2f$compiled$2f$react$2f$jsx$2d$dev$2d$runtime$2e$js__$5b$app$2d$client$5d$__$28$ecmascript$29$__["jsxDEV"])("div", {
                                                            className: `w-6 h-6 rounded-full border flex items-center justify-center transition-colors ${isSelected ? "border-[#33C5E0] bg-[#33C5E0]" : "border-[#2d3b4f] bg-white"}`,
                                                            children: isSelected && /*#__PURE__*/ (0, __TURBOPACK__imported__module__$5b$project$5d2f$node_modules$2f$next$2f$dist$2f$compiled$2f$react$2f$jsx$2d$dev$2d$runtime$2e$js__$5b$app$2d$client$5d$__$28$ecmascript$29$__["jsxDEV"])(__TURBOPACK__imported__module__$5b$project$5d2f$node_modules$2f$lucide$2d$react$2f$dist$2f$esm$2f$icons$2f$check$2e$js__$5b$app$2d$client$5d$__$28$ecmascript$29$__$3c$export__default__as__Check$3e$__["Check"], {
                                                                className: "w-4 h-4 text-black",
                                                                strokeWidth: 3
                                                            }, void 0, false, {
                                                                fileName: "[project]/components/WalletModal.tsx",
                                                                lineNumber: 76,
                                                                columnNumber: 31
                                                            }, this)
                                                        }, void 0, false, {
                                                            fileName: "[project]/components/WalletModal.tsx",
                                                            lineNumber: 72,
                                                            columnNumber: 27
                                                        }, this),
                                                        /*#__PURE__*/ (0, __TURBOPACK__imported__module__$5b$project$5d2f$node_modules$2f$next$2f$dist$2f$compiled$2f$react$2f$jsx$2d$dev$2d$runtime$2e$js__$5b$app$2d$client$5d$__$28$ecmascript$29$__["jsxDEV"])("div", {
                                                            className: "text-[#92A5A8]",
                                                            children: wallet.id.includes("freighter") ? /*#__PURE__*/ (0, __TURBOPACK__imported__module__$5b$project$5d2f$node_modules$2f$next$2f$dist$2f$compiled$2f$react$2f$jsx$2d$dev$2d$runtime$2e$js__$5b$app$2d$client$5d$__$28$ecmascript$29$__["jsxDEV"])(__TURBOPACK__imported__module__$5b$project$5d2f$node_modules$2f$lucide$2d$react$2f$dist$2f$esm$2f$icons$2f$wallet$2e$js__$5b$app$2d$client$5d$__$28$ecmascript$29$__$3c$export__default__as__Wallet$3e$__["Wallet"], {
                                                                className: "w-5 h-5"
                                                            }, void 0, false, {
                                                                fileName: "[project]/components/WalletModal.tsx",
                                                                lineNumber: 87,
                                                                columnNumber: 31
                                                            }, this) : /*#__PURE__*/ (0, __TURBOPACK__imported__module__$5b$project$5d2f$node_modules$2f$next$2f$dist$2f$compiled$2f$react$2f$jsx$2d$dev$2d$runtime$2e$js__$5b$app$2d$client$5d$__$28$ecmascript$29$__["jsxDEV"])(__TURBOPACK__imported__module__$5b$project$5d2f$node_modules$2f$lucide$2d$react$2f$dist$2f$esm$2f$icons$2f$wallet$2e$js__$5b$app$2d$client$5d$__$28$ecmascript$29$__$3c$export__default__as__Wallet$3e$__["Wallet"], {
                                                                className: "w-5 h-5"
                                                            }, void 0, false, {
                                                                fileName: "[project]/components/WalletModal.tsx",
                                                                lineNumber: 89,
                                                                columnNumber: 31
                                                            }, this)
                                                        }, void 0, false, {
                                                            fileName: "[project]/components/WalletModal.tsx",
                                                            lineNumber: 84,
                                                            columnNumber: 27
                                                        }, this),
                                                        /*#__PURE__*/ (0, __TURBOPACK__imported__module__$5b$project$5d2f$node_modules$2f$next$2f$dist$2f$compiled$2f$react$2f$jsx$2d$dev$2d$runtime$2e$js__$5b$app$2d$client$5d$__$28$ecmascript$29$__["jsxDEV"])("span", {
                                                            className: "font-semibold text-sm tracking-wider uppercase text-[#92A5A8]",
                                                            children: wallet.name
                                                        }, void 0, false, {
                                                            fileName: "[project]/components/WalletModal.tsx",
                                                            lineNumber: 93,
                                                            columnNumber: 27
                                                        }, this)
                                                    ]
                                                }, void 0, true, {
                                                    fileName: "[project]/components/WalletModal.tsx",
                                                    lineNumber: 67,
                                                    columnNumber: 25
                                                }, this)
                                            ]
                                        }, wallet.id, true, {
                                            fileName: "[project]/components/WalletModal.tsx",
                                            lineNumber: 65,
                                            columnNumber: 23
                                        }, this);
                                    })
                                }, void 0, false, {
                                    fileName: "[project]/components/WalletModal.tsx",
                                    lineNumber: 61,
                                    columnNumber: 17
                                }, this),
                                /*#__PURE__*/ (0, __TURBOPACK__imported__module__$5b$project$5d2f$node_modules$2f$next$2f$dist$2f$compiled$2f$react$2f$jsx$2d$dev$2d$runtime$2e$js__$5b$app$2d$client$5d$__$28$ecmascript$29$__["jsxDEV"])("button", {
                                    onClick: handleConnectClick,
                                    disabled: !activeSelection || isConnecting,
                                    className: `w-full py-4 rounded-full font-medium text-white transition-all flex items-center justify-center gap-2 ${activeSelection ? "bg-[#1C252A] hover:bg-[#1C252A]" : "bg-[#1C252A] cursor-not-allowed text-gray-500"}`,
                                    children: [
                                        /*#__PURE__*/ (0, __TURBOPACK__imported__module__$5b$project$5d2f$node_modules$2f$next$2f$dist$2f$compiled$2f$react$2f$jsx$2d$dev$2d$runtime$2e$js__$5b$app$2d$client$5d$__$28$ecmascript$29$__["jsxDEV"])(__TURBOPACK__imported__module__$5b$project$5d2f$components$2f$userIcon$2e$tsx__$5b$app$2d$client$5d$__$28$ecmascript$29$__["default"], {}, void 0, false, {
                                            fileName: "[project]/components/WalletModal.tsx",
                                            lineNumber: 107,
                                            columnNumber: 18
                                        }, this),
                                        /*#__PURE__*/ (0, __TURBOPACK__imported__module__$5b$project$5d2f$node_modules$2f$next$2f$dist$2f$compiled$2f$react$2f$jsx$2d$dev$2d$runtime$2e$js__$5b$app$2d$client$5d$__$28$ecmascript$29$__["jsxDEV"])("span", {
                                            children: "Connect Wallet"
                                        }, void 0, false, {
                                            fileName: "[project]/components/WalletModal.tsx",
                                            lineNumber: 108,
                                            columnNumber: 19
                                        }, this)
                                    ]
                                }, void 0, true, {
                                    fileName: "[project]/components/WalletModal.tsx",
                                    lineNumber: 102,
                                    columnNumber: 17
                                }, this)
                            ]
                        }, void 0, true, {
                            fileName: "[project]/components/WalletModal.tsx",
                            lineNumber: 51,
                            columnNumber: 15
                        }, this)
                    }, void 0, false, {
                        fileName: "[project]/components/WalletModal.tsx",
                        lineNumber: 44,
                        columnNumber: 13
                    }, this)
                }, void 0, false, {
                    fileName: "[project]/components/WalletModal.tsx",
                    lineNumber: 43,
                    columnNumber: 11
                }, this)
            ]
        }, void 0, true)
    }, void 0, false, {
        fileName: "[project]/components/WalletModal.tsx",
        lineNumber: 35,
        columnNumber: 5
    }, this);
}
_s(WalletModal, "WKWUoeM65gKQvLBnV9+9+Wdb9wI=", false, function() {
    return [
        __TURBOPACK__imported__module__$5b$project$5d2f$context$2f$WalletContext$2e$tsx__$5b$app$2d$client$5d$__$28$ecmascript$29$__["useWallet"]
    ];
});
_c = WalletModal;
var _c;
__turbopack_context__.k.register(_c, "WalletModal");
if (typeof globalThis.$RefreshHelpers$ === 'object' && globalThis.$RefreshHelpers !== null) {
    __turbopack_context__.k.registerExports(__turbopack_context__.m, globalThis.$RefreshHelpers$);
}
}),
]);

//# sourceMappingURL=_59c67d0a._.js.map