import { describe, it, afterEach } from "node:test";
import assert from "node:assert/strict";
import { Asset, Contract, Keypair, Networks } from "@stellar/stellar-sdk";
import { detectSac, sacLabel, reloadSacMap } from "../src/sac.js";

const NETWORK_PASSPHRASE = process.env.NETWORK_PASSPHRASE || Networks.TESTNET;
const NATIVE_CONTRACT_ID = new Contract(Asset.native().contractId(NETWORK_PASSPHRASE)).contractId();

const sampleIssuer = Keypair.random().publicKey();
const usdcAsset = new Asset("USDC", sampleIssuer);
const usdcContractId = new Contract(usdcAsset.contractId(NETWORK_PASSPHRASE)).contractId();

describe("sac", () => {
  const originalSacAssets = process.env.SAC_ASSETS;

  afterEach(() => {
    if (originalSacAssets !== undefined) {
      process.env.SAC_ASSETS = originalSacAssets;
    } else {
      delete process.env.SAC_ASSETS;
    }
    reloadSacMap();
  });

  it("detects native XLM SAC contract", () => {
    const res = detectSac(NATIVE_CONTRACT_ID);
    assert.deepEqual(res, { isSac: true, assetCode: "XLM" });
    assert.equal(sacLabel(NATIVE_CONTRACT_ID, "fallback"), "XLM");
  });

  it("returns isSac false for unknown contract IDs", () => {
    const unknownContract = "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAC3M";
    const res = detectSac(unknownContract);
    assert.deepEqual(res, { isSac: false, assetCode: null });
    assert.equal(sacLabel(unknownContract, "fallback"), "fallback");
  });

  it("reloads SAC map dynamically when SAC_ASSETS env var is updated", () => {
    // Before reload, USDC is not in SAC map
    assert.equal(detectSac(usdcContractId).isSac, false);

    // Update env var and reload SAC map
    process.env.SAC_ASSETS = JSON.stringify([{ code: "USDC", issuer: sampleIssuer }]);
    reloadSacMap();

    // After reload, USDC is recognised
    const res = detectSac(usdcContractId);
    assert.deepEqual(res, { isSac: true, assetCode: "USDC" });
    assert.equal(sacLabel(usdcContractId), "USDC");
  });

  it("handles malformed SAC_ASSETS JSON gracefully without throwing", () => {
    process.env.SAC_ASSETS = "invalid-json-string";
    assert.doesNotThrow(() => reloadSacMap());

    // Native XLM should still be recognised
    assert.equal(detectSac(NATIVE_CONTRACT_ID).isSac, true);
  });
});
