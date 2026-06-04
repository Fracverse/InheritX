import { ModuleInterface } from './types';
/**
 * This method returns all modules that don't require extra configuration before they can be loaded
 * You can provide a filter function if needed
 */
export declare function allowAllModules(opts?: {
    filterBy: (module: ModuleInterface) => boolean;
}): ModuleInterface[];
/**
 * This method only returns those modules from wallet that follow exactly the SEP-43 standard and don't require extra configuration before they can be loaded
 * You can provide a filter function if needed
 */
export declare function sep43Modules(opts?: {
    filterBy: (module: ModuleInterface) => boolean;
}): ModuleInterface[];
export declare function parseError(e: any): {
    code: any;
    message: any;
    ext: any;
};
//# sourceMappingURL=utils.d.ts.map