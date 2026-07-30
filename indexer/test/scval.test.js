import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { xdr } from "@stellar/stellar-sdk";
import { scValToJs } from "../src/scval.js";

describe("scValToJs", () => {
  describe("scvI128", () => {
    it("correctly handles negative hi component for -1n", () => {
      const val = xdr.ScVal.scvI128(
        new xdr.Int128Parts({
          hi: xdr.Int64.fromString("-1"),
          lo: xdr.Uint64.fromString("18446744073709551615"),
        })
      );
      assert.equal(scValToJs(val), -1n);
    });

    it("correctly reconstructs signed 128-bit value when hi is negative and lo is 0", () => {
      const val = xdr.ScVal.scvI128(
        new xdr.Int128Parts({
          hi: xdr.Int64.fromString("-1"),
          lo: xdr.Uint64.fromString("0"),
        })
      );
      assert.equal(scValToJs(val), -18446744073709551616n);
    });

    it("correctly reconstructs positive 128-bit value", () => {
      const val = xdr.ScVal.scvI128(
        new xdr.Int128Parts({
          hi: xdr.Int64.fromString("1"),
          lo: xdr.Uint64.fromString("500"),
        })
      );
      assert.equal(scValToJs(val), 18446744073709552116n);
    });
  });

  describe("scvU128", () => {
    it("correctly reconstructs max uint128 value", () => {
      const val = xdr.ScVal.scvU128(
        new xdr.UInt128Parts({
          hi: xdr.Uint64.fromString("18446744073709551615"),
          lo: xdr.Uint64.fromString("18446744073709551615"),
        })
      );
      assert.equal(scValToJs(val), 340282366920938463463374607431768211455n);
    });
  });

  describe("scvI256", () => {
    it("correctly handles negative hiHi component for -1n", () => {
      const maxU64 = "18446744073709551615";
      const val = xdr.ScVal.scvI256(
        new xdr.Int256Parts({
          hiHi: xdr.Int64.fromString("-1"),
          hiLo: xdr.Uint64.fromString(maxU64),
          loHi: xdr.Uint64.fromString(maxU64),
          loLo: xdr.Uint64.fromString(maxU64),
        })
      );
      assert.equal(scValToJs(val), -1n);
    });
  });

  describe("scvU256", () => {
    it("correctly reconstructs max uint256 value", () => {
      const maxU64 = "18446744073709551615";
      const val = xdr.ScVal.scvU256(
        new xdr.UInt256Parts({
          hiHi: xdr.Uint64.fromString(maxU64),
          hiLo: xdr.Uint64.fromString(maxU64),
          loHi: xdr.Uint64.fromString(maxU64),
          loLo: xdr.Uint64.fromString(maxU64),
        })
      );
      assert.equal(
        scValToJs(val),
        115792089237316195423570985008687907853269984665640564039457584007913129639935n
      );
    });
  });
});
