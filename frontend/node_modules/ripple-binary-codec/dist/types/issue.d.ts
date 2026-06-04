import { BinaryParser } from '../serdes/binary-parser';
import { JsonObject, SerializedType } from './serialized-type';
interface XRPIssue extends JsonObject {
    currency: string;
}
interface IOUIssue extends JsonObject {
    currency: string;
    issuer: string;
}
interface MPTIssue extends JsonObject {
    mpt_issuance_id: string;
}
/**
 * Interface for JSON objects that represent issues
 */
type IssueObject = XRPIssue | IOUIssue | MPTIssue;
/**
 * Class for serializing/Deserializing Issue
 */
declare class Issue extends SerializedType {
    static readonly XRP_ISSUE: Issue;
    constructor(bytes: Uint8Array);
    /**
     * Construct Issue from XRPIssue, IOUIssue or MPTIssue
     *
     * @param value An object representing an XRPIssue, IOUIssue or MPTIssue
     * @returns An Issue object
     */
    static from<T extends Issue | IssueObject>(value: T): Issue;
    /**
     * Read Issue from a BinaryParser
     *
     * @param parser BinaryParser to read the Issue from
     *
     * @returns An Issue object
     */
    static fromParser(parser: BinaryParser): Issue;
    /**
     * Get the JSON representation of this IssueObject
     *
     * @returns the JSON interpretation of this.bytes
     */
    toJSON(): IssueObject;
}
export { Issue, IssueObject };
