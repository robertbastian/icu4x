// generated by diplomat-tool
import { MeasureUnit } from "./MeasureUnit.mjs"
import wasm from "./diplomat-wasm.mjs";
import * as diplomatRuntime from "./diplomat-runtime.mjs";


/** 
 * An ICU4X Measurement Unit parser object which is capable of parsing the CLDR unit identifier
 * (e.g. `meter-per-square-second`) and get the [`MeasureUnit`].
 *
 * See the [Rust documentation for `MeasureUnitParser`](https://docs.rs/icu/latest/icu/experimental/measure/parser/struct.MeasureUnitParser.html) for more information.
 */
const MeasureUnitParser_box_destroy_registry = new FinalizationRegistry((ptr) => {
    wasm.icu4x_MeasureUnitParser_destroy_mv1(ptr);
});

export class MeasureUnitParser {
    
    // Internal ptr reference:
    #ptr = null;

    // Lifetimes are only to keep dependencies alive.
    // Since JS won't garbage collect until there are no incoming edges.
    #selfEdge = [];
    #aEdge = [];
    
    #internalConstructor(symbol, ptr, selfEdge, aEdge) {
        if (symbol !== diplomatRuntime.internalConstructor) {
            console.error("MeasureUnitParser is an Opaque type. You cannot call its constructor.");
            return;
        }
        
        
        this.#aEdge = aEdge;
        
        this.#ptr = ptr;
        this.#selfEdge = selfEdge;
        
        // Are we being borrowed? If not, we can register.
        if (this.#selfEdge.length === 0) {
            MeasureUnitParser_box_destroy_registry.register(this, this.#ptr);
        }
        
        return this;
    }
    get ffiValue() {
        return this.#ptr;
    }

    /** 
     * Parses the CLDR unit identifier (e.g. `meter-per-square-second`) and returns the corresponding [`MeasureUnit`],
     * if the identifier is valid.
     *
     * See the [Rust documentation for `parse`](https://docs.rs/icu/latest/icu/experimental/measure/parser/struct.MeasureUnitParser.html#method.parse) for more information.
     */
    parse(unitId) {
        let functionCleanupArena = new diplomatRuntime.CleanupArena();
        
        const unitIdSlice = functionCleanupArena.alloc(diplomatRuntime.DiplomatBuf.str8(wasm, unitId));
        
        const result = wasm.icu4x_MeasureUnitParser_parse_mv1(this.ffiValue, ...unitIdSlice.splat());
    
        try {
            return result === 0 ? null : new MeasureUnit(diplomatRuntime.internalConstructor, result, []);
        }
        
        finally {
            functionCleanupArena.free();
        }
    }

    constructor(symbol, ptr, selfEdge, aEdge) {
        return this.#internalConstructor(...arguments)
    }
}