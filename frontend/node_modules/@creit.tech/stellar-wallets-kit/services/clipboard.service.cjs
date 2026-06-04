'use strict';

async function copyToClipboard(textToCopy) {
    if (!textToCopy) {
        throw new Error(`Text to copy to the clipboard can't be undefined`);
    }
    await navigator.clipboard.writeText(textToCopy);
}

exports.copyToClipboard = copyToClipboard;
//# sourceMappingURL=clipboard.service.cjs.map
