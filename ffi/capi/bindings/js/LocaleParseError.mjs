// generated by diplomat-tool
import wasm from "./diplomat-wasm.mjs";
import * as diplomatRuntime from "./diplomat-runtime.mjs";

// Base enumerator definition
/** Additional information: [1](https://docs.rs/icu/latest/icu/locale/enum.ParseError.html)
*/
export class LocaleParseError {
    #value = undefined;

    static values = new Map([
        ["Unknown", 0],
        ["Language", 1],
        ["Subtag", 2],
        ["Extension", 3]
    ]);
    constructor(value) {
        if (value instanceof LocaleParseError) {
            this.#value = value.value;
            return;
        }

        if (LocaleParseError.values.has(value)) {
            this.#value = value;
            return;
        }

        throw TypeError(value + " is not a LocaleParseError and does not correspond to any of its enumerator values.");
    }

    get value() {
        return this.#value;
    }

    get ffiValue() {
        return LocaleParseError.values.get(this.#value);
    }

    static Unknown = new LocaleParseError("Unknown");

    static Language = new LocaleParseError("Language");

    static Subtag = new LocaleParseError("Subtag");

    static Extension = new LocaleParseError("Extension");


    

}