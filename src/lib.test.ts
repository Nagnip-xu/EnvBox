import { describe, expect, it } from "vitest";
import { translate, LANGS } from "./i18n";
import { resolveTheme, applyTheme } from "./theme";

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
