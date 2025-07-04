// generated by diplomat-tool
import { CalendarKind } from "./CalendarKind.mjs"
import wasm from "./diplomat-wasm.mjs";
import * as diplomatRuntime from "./diplomat-runtime.mjs";



/**
 * See the [Rust documentation for `MismatchedCalendarError`](https://docs.rs/icu/2.0.0/icu/datetime/struct.MismatchedCalendarError.html) for more information.
 */
export class DateTimeMismatchedCalendarError {
    #thisKind;
    get thisKind() {
        return this.#thisKind;
    }
    set thisKind(value){
        this.#thisKind = value;
    }
    #dateKind;
    get dateKind() {
        return this.#dateKind;
    }
    set dateKind(value){
        this.#dateKind = value;
    }
    /** @internal */
    static fromFields(structObj) {
        return new DateTimeMismatchedCalendarError(structObj);
    }

    #internalConstructor(structObj) {
        if (typeof structObj !== "object") {
            throw new Error("DateTimeMismatchedCalendarError's constructor takes an object of DateTimeMismatchedCalendarError's fields.");
        }

        if ("thisKind" in structObj) {
            this.#thisKind = structObj.thisKind;
        } else {
            throw new Error("Missing required field thisKind.");
        }

        if ("dateKind" in structObj) {
            this.#dateKind = structObj.dateKind;
        } else {
            this.#dateKind = null;
        }

        return this;
    }

    // Return this struct in FFI function friendly format.
    // Returns an array that can be expanded with spread syntax (...)
    _intoFFI(
        functionCleanupArena,
        appendArrayMap
    ) {
        let buffer = diplomatRuntime.DiplomatBuf.struct(wasm, 12, 4);

        this._writeToArrayBuffer(wasm.memory.buffer, buffer.ptr, functionCleanupArena, appendArrayMap);

        functionCleanupArena.alloc(buffer);

        return buffer.ptr;
    }

    static _fromSuppliedValue(internalConstructor, obj) {
        if (internalConstructor !== diplomatRuntime.internalConstructor) {
            throw new Error("_fromSuppliedValue cannot be called externally.");
        }

        if (obj instanceof DateTimeMismatchedCalendarError) {
            return obj;
        }

        return DateTimeMismatchedCalendarError.fromFields(obj);
    }

    _writeToArrayBuffer(
        arrayBuffer,
        offset,
        functionCleanupArena,
        appendArrayMap
    ) {
        diplomatRuntime.writeToArrayBuffer(arrayBuffer, offset + 0, this.#thisKind.ffiValue, Int32Array);
        diplomatRuntime.writeOptionToArrayBuffer(arrayBuffer, offset + 4, this.#dateKind, 4, 4, (arrayBuffer, offset, jsValue) => diplomatRuntime.writeToArrayBuffer(arrayBuffer, offset + 0, jsValue.ffiValue, Int32Array));
    }

    // This struct contains borrowed fields, so this takes in a list of
    // "edges" corresponding to where each lifetime's data may have been borrowed from
    // and passes it down to individual fields containing the borrow.
    // This method does not attempt to handle any dependencies between lifetimes, the caller
    // should handle this when constructing edge arrays.
    static _fromFFI(internalConstructor, ptr) {
        if (internalConstructor !== diplomatRuntime.internalConstructor) {
            throw new Error("DateTimeMismatchedCalendarError._fromFFI is not meant to be called externally. Please use the default constructor.");
        }
        let structObj = {};
        const thisKindDeref = diplomatRuntime.enumDiscriminant(wasm, ptr);
        structObj.thisKind = new CalendarKind(diplomatRuntime.internalConstructor, thisKindDeref);
        const dateKindDeref = ptr + 4;
        structObj.dateKind = diplomatRuntime.readOption(wasm, dateKindDeref, 4, (wasm, offset) => { const deref = diplomatRuntime.enumDiscriminant(wasm, offset); return new CalendarKind(diplomatRuntime.internalConstructor, deref) });

        return new DateTimeMismatchedCalendarError(structObj);
    }


    constructor(structObj) {
        return this.#internalConstructor(...arguments)
    }
}