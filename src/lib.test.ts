import { describe, expect, it } from "vitest";
import { translate, LANGS } from "./i18n";
import { resolveTheme, applyTheme } from "./theme";
import { errorCode, errorMessage } from "./lib/tauri";
import { displaySensitiveValue, isSensitiveName } from "./lib/security";

describe("i18n", () => {
  it("returns zh translation by default key", () => {
    expect(translate("zh", "nav.sdk")).toBe("SDK 中心");
  });

  it("falls back when key missing", () => {
    expect(translate("en", "missing.key", "fallback")).toBe("fallback");
  });

  it("has four languages", () => {
    expect(LANGS).toHaveLength(4);
  });

  it("interpolates params", () => {
    expect(translate("en", "toast.imported", { n: 3 })).toBe("Imported 3 variables");
    expect(translate("zh", "toast.varDeleted", { name: "JAVA_HOME" })).toBe("已删除 JAVA_HOME");
  });
});

describe("theme", () => {
  it("resolves light and dark modes", () => {
    expect(resolveTheme("light")).toBe("light");
    expect(resolveTheme("dark")).toBe("dark");
  });

  it("applyTheme sets document class", () => {
    applyTheme("light");
    expect(document.documentElement.classList.contains("theme-light")).toBe(true);
    applyTheme("dark");
    expect(document.documentElement.classList.contains("theme-dark")).toBe(true);
  });
});

describe("sdkCache", () => {
  it("exports cache helpers", async () => {
    const mod = await import("./store/sdkCache");
    expect(mod.getCachedSdks()).toBeNull();
    expect(mod.isSdkScanning()).toBe(false);
  });
});

describe("tauri error formatting", () => {
  it("normalizes strings and Error objects", () => {
    expect(errorMessage("permission denied")).toBe("permission denied");
    expect(errorMessage(new Error("snapshot failed"))).toBe("snapshot failed");
    expect(errorCode({ code: "PERMISSION_DENIED", message: "denied" })).toBe(
      "PERMISSION_DENIED"
    );
  });
});

describe("sensitive values", () => {
  it("detects and masks common secret names", () => {
    expect(isSensitiveName("GITHUB_TOKEN")).toBe(true);
    expect(isSensitiveName("JAVA_HOME")).toBe(false);
    expect(displaySensitiveValue("secret", true)).toBe("••••••••••••");
  });
});
