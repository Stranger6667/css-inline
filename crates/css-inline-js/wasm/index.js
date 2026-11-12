"use strict";
var __defProp = Object.defineProperty;
var __getOwnPropDesc = Object.getOwnPropertyDescriptor;
var __getOwnPropNames = Object.getOwnPropertyNames;
var __hasOwnProp = Object.prototype.hasOwnProperty;
var __export = (target, all) => {
  for (var name in all)
    __defProp(target, name, { get: all[name], enumerable: true });
};
var __copyProps = (to, from, except, desc) => {
  if (from && typeof from === "object" || typeof from === "function") {
    for (let key of __getOwnPropNames(from))
      if (!__hasOwnProp.call(to, key) && key !== except)
        __defProp(to, key, { get: () => from[key], enumerable: !(desc = __getOwnPropDesc(from, key)) || desc.enumerable });
  }
  return to;
};
var __toCommonJS = (mod) => __copyProps(__defProp({}, "__esModule", { value: true }), mod);

// wasm-binding.ts
var wasm_binding_exports = {};
__export(wasm_binding_exports, {
  initWasm: () => initWasm,
  inline: () => inline2,
  inlineAsync: () => inlineAsync2,
  inlineFragment: () => inlineFragment2,
  inlineFragmentAsync: () => inlineFragmentAsync2,
  version: () => version2
});
module.exports = __toCommonJS(wasm_binding_exports);

// wasm/dist/index.js
function inline(html, options) {
  let deferred3_0;
  let deferred3_1;
  try {
    const ptr0 = passStringToWasm0(html, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.inline(ptr0, len0, options);
    var ptr2 = ret[0];
    var len2 = ret[1];
    if (ret[3]) {
      ptr2 = 0;
      len2 = 0;
      throw takeFromExternrefTable0(ret[2]);
    }
    deferred3_0 = ptr2;
    deferred3_1 = len2;
    return getStringFromWasm0(ptr2, len2);
  } finally {
    wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
  }
}
function inlineAsync(html, options) {
  const ptr0 = passStringToWasm0(html, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
  const len0 = WASM_VECTOR_LEN;
  const ret = wasm.inlineAsync(ptr0, len0, options);
  return ret;
}
function inlineFragment(html, css, options) {
  let deferred4_0;
  let deferred4_1;
  try {
    let ptr0, len0, ptr1, len1;
    try {
      ptr0 = passStringToWasm0(html, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      len0 = WASM_VECTOR_LEN;
      ptr1 = passStringToWasm0(css, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      len1 = WASM_VECTOR_LEN;
    } catch (e) {
      if (ptr1) {
        wasm.__wbindgen_free(ptr1, len1 * 1, 1);
      }
      if (ptr0) {
        wasm.__wbindgen_free(ptr0, len0 * 1, 1);
      }
      throw e;
    }
    const ret = wasm.inlineFragment(ptr0, len0, ptr1, len1, options);
    var ptr3 = ret[0];
    var len3 = ret[1];
    if (ret[3]) {
      ptr3 = 0;
      len3 = 0;
      throw takeFromExternrefTable0(ret[2]);
    }
    deferred4_0 = ptr3;
    deferred4_1 = len3;
    return getStringFromWasm0(ptr3, len3);
  } finally {
    wasm.__wbindgen_free(deferred4_0, deferred4_1, 1);
  }
}
function inlineFragmentAsync(html, css, options) {
  let ptr0, len0, ptr1, len1;
  try {
    ptr0 = passStringToWasm0(html, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    len0 = WASM_VECTOR_LEN;
    ptr1 = passStringToWasm0(css, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    len1 = WASM_VECTOR_LEN;
  } catch (e) {
    if (ptr1) {
      wasm.__wbindgen_free(ptr1, len1 * 1, 1);
    }
    if (ptr0) {
      wasm.__wbindgen_free(ptr0, len0 * 1, 1);
    }
    throw e;
  }
  const ret = wasm.inlineFragmentAsync(ptr0, len0, ptr1, len1, options);
  return ret;
}
function version() {
  let deferred1_0;
  let deferred1_1;
  try {
    const ret = wasm.version();
    deferred1_0 = ret[0];
    deferred1_1 = ret[1];
    return getStringFromWasm0(ret[0], ret[1]);
  } finally {
    wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
  }
}
function __wbg_get_imports() {
  const import0 = {
    __proto__: null,
    __wbg_Error_208446807d0d1e30: function(arg0, arg1) {
      const ret = Error(getStringFromWasm0(arg0, arg1));
      return ret;
    },
    __wbg_Number_ff192a72a0dcdf6d: function(arg0) {
      const ret = Number(arg0);
      return ret;
    },
    __wbg___wbindgen_boolean_get_4f1baf9a00be3c74: function(arg0) {
      const v = arg0;
      const ret = typeof v === "boolean" ? v : void 0;
      return isLikeNone(ret) ? 16777215 : ret ? 1 : 0;
    },
    __wbg___wbindgen_debug_string_f249a2a76caa9a55: function(arg0, arg1) {
      const ret = debugString(arg1);
      const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len1 = WASM_VECTOR_LEN;
      getDataViewMemory0().setInt32((arg0 >>> 0) + 4 * 1, len1, true);
      getDataViewMemory0().setInt32((arg0 >>> 0) + 4 * 0, ptr1, true);
    },
    __wbg___wbindgen_in_3f58a9d3b3a7eed2: function(arg0, arg1) {
      const ret = arg0 in arg1;
      return ret;
    },
    __wbg___wbindgen_is_object_8b968c3ab8e96595: function(arg0) {
      const val = arg0;
      const ret = typeof val === "object" && val !== null;
      return ret;
    },
    __wbg___wbindgen_is_undefined_be5b34249ff1fc15: function(arg0) {
      const ret = arg0 === void 0;
      return ret;
    },
    __wbg___wbindgen_jsval_loose_eq_b2f214e2e4b30e36: function(arg0, arg1) {
      const ret = arg0 == arg1;
      return ret;
    },
    __wbg___wbindgen_number_get_934b640d0889af78: function(arg0, arg1) {
      const obj = arg1;
      const ret = typeof obj === "number" ? obj : void 0;
      getDataViewMemory0().setFloat64((arg0 >>> 0) + 8 * 1, isLikeNone(ret) ? 0 : ret, true);
      getDataViewMemory0().setInt32((arg0 >>> 0) + 4 * 0, !isLikeNone(ret), true);
    },
    __wbg___wbindgen_string_get_7ce8eee7fd327545: function(arg0, arg1) {
      const obj = arg1;
      const ret = typeof obj === "string" ? obj : void 0;
      var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len1 = WASM_VECTOR_LEN;
      getDataViewMemory0().setInt32((arg0 >>> 0) + 4 * 1, len1, true);
      getDataViewMemory0().setInt32((arg0 >>> 0) + 4 * 0, ptr1, true);
    },
    __wbg___wbindgen_throw_4c88d8bf264fd984: function(arg0, arg1) {
      throw new Error(getStringFromWasm0(arg0, arg1));
    },
    __wbg__wbg_cb_unref_09736879a3f67559: function(arg0) {
      arg0._wbg_cb_unref();
    },
    __wbg_abort_d3b62653236ec492: function(arg0, arg1) {
      arg0.abort(arg1);
    },
    __wbg_abort_f8d1589eb0c65ee2: function(arg0) {
      arg0.abort();
    },
    __wbg_append_ba9a4dc30a0323fd: function() {
      return handleError(function(arg0, arg1, arg2, arg3, arg4) {
        arg0.append(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
      }, arguments);
    },
    __wbg_call_a410eb4b59ed5965: function() {
      return handleError(function(arg0, arg1, arg2) {
        const ret = arg0.call(arg1, arg2);
        return ret;
      }, arguments);
    },
    __wbg_clearTimeout_1daad5c6ee4fcd3b: function(arg0) {
      const ret = clearTimeout(arg0);
      return ret;
    },
    __wbg_done_57e08d0be6e30c02: function(arg0) {
      const ret = arg0.done;
      return ret;
    },
    __wbg_entries_fb0b3ba27a6e0bf7: function(arg0) {
      const ret = arg0.entries();
      return ret;
    },
    __wbg_fetch_41d29a2b2428e0c7: function(arg0, arg1) {
      const ret = arg0.fetch(arg1);
      return ret;
    },
    __wbg_fetch_7a37768ab67c5bd8: function(arg0) {
      const ret = fetch(arg0);
      return ret;
    },
    __wbg_get_d7176cfb76d71114: function(arg0, arg1) {
      const ret = arg0[arg1 >>> 0];
      return ret;
    },
    __wbg_get_with_ref_key_fc601247060b7204: function(arg0, arg1) {
      const ret = arg0[arg1];
      return ret;
    },
    __wbg_has_2c4d5cf25ec8c1cf: function() {
      return handleError(function(arg0, arg1) {
        const ret = Reflect.has(arg0, arg1);
        return ret;
      }, arguments);
    },
    __wbg_headers_5fc0106c076eb731: function(arg0) {
      const ret = arg0.headers;
      return ret;
    },
    __wbg_instanceof_ArrayBuffer_12cdaa4161423e6b: function(arg0) {
      let result;
      try {
        result = arg0 instanceof ArrayBuffer;
      } catch (_) {
        result = false;
      }
      const ret = result;
      return ret;
    },
    __wbg_instanceof_Response_5b0b4f7f1daeae51: function(arg0) {
      let result;
      try {
        result = arg0 instanceof Response;
      } catch (_) {
        result = false;
      }
      const ret = result;
      return ret;
    },
    __wbg_instanceof_Uint8Array_d7aeb0dcb3108fea: function(arg0) {
      let result;
      try {
        result = arg0 instanceof Uint8Array;
      } catch (_) {
        result = false;
      }
      const ret = result;
      return ret;
    },
    __wbg_isArray_bcf1376d0d732d08: function(arg0) {
      const ret = Array.isArray(arg0);
      return ret;
    },
    __wbg_isSafeInteger_1a35ba4d31e71d38: function(arg0) {
      const ret = Number.isSafeInteger(arg0);
      return ret;
    },
    __wbg_length_186163212ffe79c3: function(arg0) {
      const ret = arg0.length;
      return ret;
    },
    __wbg_new_13de428334042d26: function(arg0) {
      const ret = new Uint8Array(arg0);
      return ret;
    },
    __wbg_new_277c27c26f94cbc2: function() {
      return handleError(function() {
        const ret = new Headers();
        return ret;
      }, arguments);
    },
    __wbg_new_5f28518387ad0e34: function() {
      return handleError(function() {
        const ret = new AbortController();
        return ret;
      }, arguments);
    },
    __wbg_new_ba40181b3f24ac80: function(arg0, arg1) {
      try {
        var state0 = { a: arg0, b: arg1 };
        var cb0 = (arg02, arg12) => {
          const a = state0.a;
          state0.a = 0;
          try {
            return wasm_bindgen_c2b3d4a3d419e85d___convert__closures_____invoke___js_sys_212260353cf572f6___Function_fn_wasm_bindgen_c2b3d4a3d419e85d___JsValue_____wasm_bindgen_c2b3d4a3d419e85d___sys__Undefined___js_sys_212260353cf572f6___Function_fn_wasm_bindgen_c2b3d4a3d419e85d___JsValue_____wasm_bindgen_c2b3d4a3d419e85d___sys__Undefined_______true_(a, state0.b, arg02, arg12);
          } finally {
            state0.a = a;
          }
        };
        const ret = new Promise(cb0);
        return ret;
      } finally {
        state0.a = 0;
      }
    },
    __wbg_new_cd4348290afae586: function() {
      const ret = new Object();
      return ret;
    },
    __wbg_new_from_slice_b8daf545a41c6aa0: function(arg0, arg1) {
      const ret = new Uint8Array(getArrayU8FromWasm0(arg0, arg1));
      return ret;
    },
    __wbg_new_with_str_and_init_d9f6b5611184e16b: function() {
      return handleError(function(arg0, arg1, arg2) {
        const ret = new Request(getStringFromWasm0(arg0, arg1), arg2);
        return ret;
      }, arguments);
    },
    __wbg_next_af5ac6a08bb0a717: function() {
      return handleError(function(arg0) {
        const ret = arg0.next();
        return ret;
      }, arguments);
    },
    __wbg_prototypesetcall_13162965d1fe7aa3: function(arg0, arg1, arg2) {
      Uint8Array.prototype.set.call(getArrayU8FromWasm0(arg0, arg1), arg2);
    },
    __wbg_resolve_dbf78ab4942478b0: function(arg0) {
      const ret = Promise.resolve(arg0);
      return ret;
    },
    __wbg_setTimeout_b4a9096784635514: function(arg0, arg1) {
      const ret = setTimeout(arg0, arg1);
      return ret;
    },
    __wbg_set_body_877d5251e78ca815: function(arg0, arg1) {
      arg0.body = arg1;
    },
    __wbg_set_cache_7a23201e29cbd632: function(arg0, arg1) {
      arg0.cache = __wbindgen_enum_RequestCache[arg1];
    },
    __wbg_set_credentials_bb5277b4006149d4: function(arg0, arg1) {
      arg0.credentials = __wbindgen_enum_RequestCredentials[arg1];
    },
    __wbg_set_headers_6d75a7df4eb22716: function(arg0, arg1) {
      arg0.headers = arg1;
    },
    __wbg_set_method_639e69af35614d36: function(arg0, arg1, arg2) {
      arg0.method = getStringFromWasm0(arg1, arg2);
    },
    __wbg_set_mode_07e051ad8b4e2126: function(arg0, arg1) {
      arg0.mode = __wbindgen_enum_RequestMode[arg1];
    },
    __wbg_set_referrer_413a47ffe5e2e157: function(arg0, arg1, arg2) {
      arg0.referrer = getStringFromWasm0(arg1, arg2);
    },
    __wbg_set_referrer_policy_f3e837a21403cf06: function(arg0, arg1) {
      arg0.referrerPolicy = __wbindgen_enum_ReferrerPolicy[arg1];
    },
    __wbg_set_signal_0b2eee78907e5ba7: function(arg0, arg1) {
      arg0.signal = arg1;
    },
    __wbg_signal_fac908876f45725d: function(arg0) {
      const ret = arg0.signal;
      return ret;
    },
    __wbg_static_accessor_GLOBAL_THIS_003ee833af9699fc: function() {
      const ret = typeof globalThis === "undefined" ? void 0 : globalThis;
      return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    },
    __wbg_static_accessor_GLOBAL_e4293fb22cbe77da: function() {
      const ret = typeof global === "undefined" ? void 0 : global;
      return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    },
    __wbg_static_accessor_SELF_53bb20457401453d: function() {
      const ret = typeof self === "undefined" ? void 0 : self;
      return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    },
    __wbg_static_accessor_WINDOW_b5e0810bfecb7feb: function() {
      const ret = typeof window === "undefined" ? void 0 : window;
      return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    },
    __wbg_status_cb85b6c8d1769fa1: function(arg0) {
      const ret = arg0.status;
      return ret;
    },
    __wbg_text_5240eb2c41de8292: function() {
      return handleError(function(arg0) {
        const ret = arg0.text();
        return ret;
      }, arguments);
    },
    __wbg_then_3a4f899454bcf8c9: function(arg0, arg1) {
      const ret = arg0.then(arg1);
      return ret;
    },
    __wbg_then_7ede85da03ac5cd0: function(arg0, arg1, arg2) {
      const ret = arg0.then(arg1, arg2);
      return ret;
    },
    __wbg_url_9fadea28ec624f1d: function(arg0, arg1) {
      const ret = arg1.url;
      const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len1 = WASM_VECTOR_LEN;
      getDataViewMemory0().setInt32((arg0 >>> 0) + 4 * 1, len1, true);
      getDataViewMemory0().setInt32((arg0 >>> 0) + 4 * 0, ptr1, true);
    },
    __wbg_value_e0f328965e184931: function(arg0) {
      const ret = arg0.value;
      return ret;
    },
    __wbindgen_generic_0000000000000001: function(arg0, arg1) {
      const ret = makeMutClosure(arg0, arg1, wasm_bindgen_c2b3d4a3d419e85d___convert__closures_____invoke___wasm_bindgen_c2b3d4a3d419e85d___JsValue______true_);
      return ret;
    },
    __wbindgen_generic_0000000000000002: function(arg0, arg1) {
      const ret = makeMutClosure(arg0, arg1, wasm_bindgen_c2b3d4a3d419e85d___convert__closures_____invoke_______true_);
      return ret;
    },
    __wbindgen_generic_0000000000000003: function(arg0, arg1) {
      const ret = getStringFromWasm0(arg0, arg1);
      return ret;
    },
    __wbindgen_init_externref_table: function() {
      const table = wasm.__wbindgen_externrefs;
      table.set(0, void 0);
      table.set(1024, void 0);
      table.set(1025, null);
      table.set(1026, true);
      table.set(1027, false);
    }
  };
  return {
    __proto__: null,
    "./index_bg.js": import0
  };
}
function wasm_bindgen_c2b3d4a3d419e85d___convert__closures_____invoke_______true_(arg0, arg1) {
  wasm.wasm_bindgen_c2b3d4a3d419e85d___convert__closures_____invoke_______true_(arg0, arg1);
}
function wasm_bindgen_c2b3d4a3d419e85d___convert__closures_____invoke___wasm_bindgen_c2b3d4a3d419e85d___JsValue______true_(arg0, arg1, arg2) {
  wasm.wasm_bindgen_c2b3d4a3d419e85d___convert__closures_____invoke___wasm_bindgen_c2b3d4a3d419e85d___JsValue______true_(arg0, arg1, arg2);
}
function wasm_bindgen_c2b3d4a3d419e85d___convert__closures_____invoke___js_sys_212260353cf572f6___Function_fn_wasm_bindgen_c2b3d4a3d419e85d___JsValue_____wasm_bindgen_c2b3d4a3d419e85d___sys__Undefined___js_sys_212260353cf572f6___Function_fn_wasm_bindgen_c2b3d4a3d419e85d___JsValue_____wasm_bindgen_c2b3d4a3d419e85d___sys__Undefined_______true_(arg0, arg1, arg2, arg3) {
  wasm.wasm_bindgen_c2b3d4a3d419e85d___convert__closures_____invoke___js_sys_212260353cf572f6___Function_fn_wasm_bindgen_c2b3d4a3d419e85d___JsValue_____wasm_bindgen_c2b3d4a3d419e85d___sys__Undefined___js_sys_212260353cf572f6___Function_fn_wasm_bindgen_c2b3d4a3d419e85d___JsValue_____wasm_bindgen_c2b3d4a3d419e85d___sys__Undefined_______true_(arg0, arg1, arg2, arg3);
}
var __wbindgen_enum_ReferrerPolicy = ["", "no-referrer", "no-referrer-when-downgrade", "origin", "origin-when-cross-origin", "unsafe-url", "same-origin", "strict-origin", "strict-origin-when-cross-origin"];
var __wbindgen_enum_RequestCache = ["default", "no-store", "reload", "no-cache", "force-cache", "only-if-cached"];
var __wbindgen_enum_RequestCredentials = ["omit", "same-origin", "include"];
var __wbindgen_enum_RequestMode = ["same-origin", "no-cors", "cors", "navigate"];
function addToExternrefTable0(obj) {
  const idx = wasm.__externref_table_alloc();
  wasm.__wbindgen_externrefs.set(idx, obj);
  return idx;
}
var CLOSURE_DTORS = typeof FinalizationRegistry === "undefined" ? { register: () => {
}, unregister: () => {
} } : new FinalizationRegistry((state) => wasm.__wbindgen_destroy_closure(state.a, state.b));
function debugString(val) {
  switch (typeof val) {
    // primitive types
    case "number":
    case "boolean":
    case "undefined":
      return `${val}`;
    case "string":
      return JSON.stringify(val);
    case "bigint":
      return `${val}n`;
    case "symbol":
      return val.description !== void 0 ? `Symbol(${val.description})` : "Symbol";
    case "function":
      return typeof val.name === "string" && val.name !== "" ? `Function(${val.name})` : "Function";
    case "object": {
      if (val === null) {
        return "null";
      }
      if (Array.isArray(val)) {
        return "[" + Array.from(val, debugString).join(", ") + "]";
      }
      if (val instanceof Error) {
        return `${val.name}: ${val.message}
${val.stack}`;
      }
      const toStringOutput = toString.call(val);
      const className = /\[object ([^\]]+)\]/.exec(toStringOutput)?.[1];
      if (className === void 0) {
        return toStringOutput;
      }
      if (className === "Object") {
        try {
          return "Object(" + JSON.stringify(val) + ")";
        } catch (_) {
          return "Object";
        }
      }
      return className;
    }
  }
}
function getArrayU8FromWasm0(ptr, len) {
  ptr = ptr >>> 0;
  return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}
var cachedDataViewMemory0 = null;
function getDataViewMemory0() {
  if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || cachedDataViewMemory0.buffer.detached === void 0 && cachedDataViewMemory0.buffer !== wasm.memory.buffer) {
    cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
  }
  return cachedDataViewMemory0;
}
function getStringFromWasm0(ptr, len) {
  ptr = ptr >>> 0;
  return decodeText(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}
var cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
  if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
    cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
  }
  return cachedUint8ArrayMemory0;
}
function handleError(f, args) {
  try {
    return f.apply(this, args);
  } catch (e) {
    const idx = addToExternrefTable0(e);
    wasm.__wbindgen_exn_store(idx);
  }
}
function isLikeNone(x) {
  return x === void 0 || x === null;
}
function makeMutClosure(arg0, arg1, f) {
  const state = { a: arg0, b: arg1, cnt: 1 };
  const real = (...args) => {
    state.cnt++;
    const a = state.a;
    state.a = 0;
    try {
      return f(a, state.b, ...args);
    } finally {
      state.a = a;
      real._wbg_cb_unref();
    }
  };
  real._wbg_cb_unref = () => {
    if (--state.cnt === 0) {
      wasm.__wbindgen_destroy_closure(state.a, state.b);
      state.a = 0;
      CLOSURE_DTORS.unregister(state);
    }
  };
  CLOSURE_DTORS.register(real, state, state);
  return real;
}
function passStringToWasm0(arg, malloc, realloc) {
  if (realloc === void 0) {
    const buf = cachedTextEncoder.encode(arg);
    const ptr2 = malloc(buf.length, 1) >>> 0;
    getUint8ArrayMemory0().subarray(ptr2, ptr2 + buf.length).set(buf);
    WASM_VECTOR_LEN = buf.length;
    return ptr2;
  }
  let len = arg.length;
  let ptr = malloc(len, 1) >>> 0;
  const mem = getUint8ArrayMemory0();
  let offset = 0;
  for (; offset < len; offset++) {
    const code = arg.charCodeAt(offset);
    if (code > 127) break;
    mem[ptr + offset] = code;
  }
  if (offset !== len) {
    if (offset !== 0) {
      arg = arg.slice(offset);
    }
    ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
    const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
    const ret = cachedTextEncoder.encodeInto(arg, view);
    offset += ret.written;
    ptr = realloc(ptr, len, offset, 1) >>> 0;
  }
  WASM_VECTOR_LEN = offset;
  return ptr;
}
function takeFromExternrefTable0(idx) {
  const value = wasm.__wbindgen_externrefs.get(idx);
  wasm.__externref_table_dealloc(idx);
  return value;
}
var cachedTextDecoder = new TextDecoder("utf-8", { ignoreBOM: true });
cachedTextDecoder.decode();
var MAX_SAFARI_DECODE_BYTES = 2146435072;
var numBytesDecoded = 0;
function decodeText(bytes) {
  numBytesDecoded += bytes.length;
  if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
    cachedTextDecoder = new TextDecoder("utf-8", { ignoreBOM: true });
    cachedTextDecoder.decode();
    numBytesDecoded = bytes.length;
  }
  return cachedTextDecoder.decode(bytes);
}
var cachedTextEncoder = new TextEncoder();
if (!("encodeInto" in cachedTextEncoder)) {
  cachedTextEncoder.encodeInto = function(arg, view) {
    const buf = cachedTextEncoder.encode(arg);
    view.set(buf);
    return {
      read: arg.length,
      written: buf.length
    };
  };
}
var WASM_VECTOR_LEN = 0;
var wasmModule;
var wasmInstance;
var wasm;
function __wbg_finalize_init(instance, module2) {
  wasmInstance = instance;
  wasm = instance.exports;
  wasmModule = module2;
  cachedDataViewMemory0 = null;
  cachedUint8ArrayMemory0 = null;
  wasm.__wbindgen_start();
  return wasm;
}
async function __wbg_load(module2, imports) {
  if (typeof Response === "function" && module2 instanceof Response) {
    if (!module2.ok) {
      throw new Error(`failed to fetch Wasm: ${module2.status} ${module2.statusText} fetching '${module2.url}'`);
    }
    if (typeof WebAssembly.instantiateStreaming === "function") {
      try {
        return await WebAssembly.instantiateStreaming(module2, imports);
      } catch (e) {
        const validResponse = expectedResponseType(module2.type);
        if (validResponse && module2.headers.get("Content-Type") !== "application/wasm") {
          console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);
        } else {
          throw e;
        }
      }
    }
    const bytes = await module2.arrayBuffer();
    return await WebAssembly.instantiate(bytes, imports);
  } else {
    const instance = await WebAssembly.instantiate(module2, imports);
    if (instance instanceof WebAssembly.Instance) {
      return { instance, module: module2 };
    } else {
      return instance;
    }
  }
  function expectedResponseType(type) {
    switch (type) {
      case "basic":
      case "cors":
      case "default":
        return true;
    }
    return false;
  }
}
var __wbg_init_promise;
async function __wbg_init(module_or_path) {
  if (wasm !== void 0) return wasm;
  if (__wbg_init_promise === void 0) {
    __wbg_init_promise = __wbg_init_once(module_or_path);
    __wbg_init_promise.catch(() => {
      __wbg_init_promise = void 0;
    });
  }
  return __wbg_init_promise;
}
async function __wbg_init_once(module_or_path) {
  if (module_or_path !== void 0) {
    if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
      ({ module_or_path } = module_or_path);
    } else {
      console.warn("using deprecated parameters for the initialization function; pass a single object instead");
    }
  }
  if (module_or_path === void 0) {
    module_or_path = new URL("index_bg.wasm", void 0);
  }
  const imports = __wbg_get_imports();
  if (typeof module_or_path === "string" || typeof Request === "function" && module_or_path instanceof Request || typeof URL === "function" && module_or_path instanceof URL) {
    module_or_path = fetch(module_or_path);
  }
  const { instance, module: module2 } = await __wbg_load(await module_or_path, imports);
  return __wbg_finalize_init(instance, module2);
}

// wasm-binding.ts
var initialized = false;
var initWasm = async (module_or_path) => {
  if (initialized) {
    throw new Error(
      "Already initialized. The `initWasm()` function can be used only once."
    );
  }
  await __wbg_init(await module_or_path);
  initialized = true;
};
function inline2(html, options) {
  return inline(html, options);
}
function inlineFragment2(html, css, options) {
  return inlineFragment(html, css, options);
}
function inlineAsync2(html, options) {
  return inlineAsync(html, options);
}
function inlineFragmentAsync2(html, css, options) {
  return inlineFragmentAsync(html, css, options);
}
function version2() {
  return version();
}
