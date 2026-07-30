import { describe, it } from "node:test";
import assert from "node:assert/strict";

// The replacer used in decoder.js for raw_data serialisation.
// Defined here to test it independently of the DB-coupled decode() function.
const bigIntReplacer = (_, v) => (typeof v === "bigint" ? v.toString() : v);

describe("decoder raw_data BigInt serialisation", () => {
  it("serialises a top-level BigInt without throwing", () => {
    const data = 1000000000000n;
    assert.doesNotThrow(() => JSON.stringify(data, bigIntReplacer));
    assert.equal(JSON.stringify(data, bigIntReplacer), '"1000000000000"');
  });

  it("serialises an object with a BigInt value field (i128 shape)", () => {
    // scValToNative returns BigInt for i128; simulate that here
    const data = { amount: 99999999999999999999n };
    assert.doesNotThrow(() => JSON.stringify(data, bigIntReplacer));
    const parsed = JSON.parse(JSON.stringify(data, bigIntReplacer));
    assert.equal(parsed.amount, "99999999999999999999");
  });

  it("serialises a nested array containing BigInts (vec of i64)", () => {
    const data = [1n, 2n, 3n];
    assert.doesNotThrow(() => JSON.stringify(data, bigIntReplacer));
    const parsed = JSON.parse(JSON.stringify(data, bigIntReplacer));
    assert.deepEqual(parsed, ["1", "2", "3"]);
  });

  it("leaves non-BigInt values unchanged", () => {
    const data = { str: "hello", num: 42, flag: true, nothing: null };
    const result = JSON.parse(JSON.stringify(data, bigIntReplacer));
    assert.deepEqual(result, data);
  });

  it("plain JSON.stringify throws on BigInt (confirms the bug existed)", () => {
    assert.throws(
      () => JSON.stringify(99n),
      /BigInt/
    );
  });
});
