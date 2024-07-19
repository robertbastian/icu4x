// generated by diplomat-tool
import wasm from "./diplomat-wasm.mjs";
import * as diplomatRuntime from "./diplomat-runtime.mjs";

// Base enumerator definition
/** Mode used in a rounding operation.
*
*See the [Rust documentation for `RoundingMode`](https://docs.rs/fixed_decimal/latest/fixed_decimal/enum.RoundingMode.html) for more information.
*/
export class FixedDecimalRoundingMode {
    #value = undefined;

    static values = new Map([
        ["Ceil", 0],
        ["Expand", 1],
        ["Floor", 2],
        ["Trunc", 3],
        ["HalfCeil", 4],
        ["HalfExpand", 5],
        ["HalfFloor", 6],
        ["HalfTrunc", 7],
        ["HalfEven", 8]
    ]);
    constructor(value) {
        if (value instanceof FixedDecimalRoundingMode) {
            this.#value = value.value;
            return;
        }

        if (FixedDecimalRoundingMode.values.has(value)) {
            this.#value = value;
            return;
        }

        throw TypeError(value + " is not a FixedDecimalRoundingMode and does not correspond to any of its enumerator values.");
    }

    get value() {
        return this.#value;
    }

    get ffiValue() {
        return FixedDecimalRoundingMode.values.get(this.#value);
    }

    static Ceil = new FixedDecimalRoundingMode("Ceil");

    static Expand = new FixedDecimalRoundingMode("Expand");

    static Floor = new FixedDecimalRoundingMode("Floor");

    static Trunc = new FixedDecimalRoundingMode("Trunc");

    static HalfCeil = new FixedDecimalRoundingMode("HalfCeil");

    static HalfExpand = new FixedDecimalRoundingMode("HalfExpand");

    static HalfFloor = new FixedDecimalRoundingMode("HalfFloor");

    static HalfTrunc = new FixedDecimalRoundingMode("HalfTrunc");

    static HalfEven = new FixedDecimalRoundingMode("HalfEven");


    

}