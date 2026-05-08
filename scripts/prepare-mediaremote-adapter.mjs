#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import { cpSync, existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const REPO_URL = "https://github.com/ungive/mediaremote-adapter.git";
const REPO_TAG = "v0.7.5";
const REPO_COMMIT = "cc5d90aa0b633acc34a8c44197b5fcb309958935";
const VERSION_SIGNATURE = `${REPO_URL}@${REPO_TAG}#${REPO_COMMIT}`;

const scriptDir = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(scriptDir, "..");
const tauriRoot = join(repoRoot, "src-tauri");
const cacheRoot = join(tauriRoot, ".cache", "mediaremote-adapter");
const sourceRoot = join(cacheRoot, "source");
const buildRoot = join(cacheRoot, "build");
const resourceRoot = join(tauriRoot, "resources", "mediaremote-adapter");
const versionFile = join(resourceRoot, ".source-version");
const scriptPath = join(resourceRoot, "mediaremote-adapter.pl");
const frameworkDir = join(resourceRoot, "MediaRemoteAdapter.framework");
const frameworkBinary = join(frameworkDir, "MediaRemoteAdapter");

function run(program, args, options = {}) {
  const result = spawnSync(program, args, {
    cwd: options.cwd ?? repoRoot,
    encoding: "utf8",
    stdio: options.capture ? ["ignore", "pipe", "pipe"] : "inherit",
  });

  if (result.error) {
    throw new Error(`${program} 执行失败：${result.error.message}`);
  }

  if (result.status !== 0) {
    const stderr = (result.stderr ?? "").trim();
    const stdout = (result.stdout ?? "").trim();
    const detail = stderr || stdout || `exit code ${result.status}`;
    throw new Error(`${program} 执行失败：${detail}`);
  }

  return result.stdout ?? "";
}

function ensureDirectory(path) {
  mkdirSync(path, { recursive: true });
}

function isPrepared() {
  return (
    existsSync(scriptPath) &&
    existsSync(frameworkBinary) &&
    existsSync(versionFile) &&
    readFileSync(versionFile, "utf8").trim() === VERSION_SIGNATURE
  );
}

function ensureSourceCheckout() {
  const headPath = join(sourceRoot, ".git", "HEAD");
  if (existsSync(headPath)) {
    const currentHead = run("git", ["-C", sourceRoot, "rev-parse", "HEAD"], {
      capture: true,
    }).trim();
    if (currentHead === REPO_COMMIT) {
      return;
    }
  }

  rmSync(sourceRoot, { recursive: true, force: true });
  ensureDirectory(cacheRoot);
  run(
    "git",
    ["clone", "--depth", "1", "--branch", REPO_TAG, REPO_URL, sourceRoot],
  );

  const currentHead = run("git", ["-C", sourceRoot, "rev-parse", "HEAD"], {
    capture: true,
  }).trim();
  if (currentHead !== REPO_COMMIT) {
    throw new Error(
      `mediaremote-adapter 版本不匹配：当前 ${currentHead}，期望 ${REPO_COMMIT}`,
    );
  }
}

function buildAdapter() {
  ensureDirectory(buildRoot);
  run("cmake", ["-S", sourceRoot, "-B", buildRoot]);
  run("cmake", ["--build", buildRoot, "--target", "MediaRemoteAdapter"]);
}

function refreshBundleResources() {
  ensureDirectory(resourceRoot);
  rmSync(scriptPath, { force: true });
  rmSync(frameworkDir, { recursive: true, force: true });
  rmSync(versionFile, { force: true });

  cpSync(join(sourceRoot, "bin", "mediaremote-adapter.pl"), scriptPath);
  cpSync(join(buildRoot, "MediaRemoteAdapter.framework"), frameworkDir, {
    recursive: true,
  });
  writeFileSync(versionFile, `${VERSION_SIGNATURE}\n`, "utf8");
}

function main() {
  if (process.platform !== "darwin") {
    return;
  }

  if (isPrepared()) {
    return;
  }

  ensureSourceCheckout();
  buildAdapter();
  refreshBundleResources();
}

try {
  main();
} catch (error) {
  console.error(
    `[mediaremote-adapter] ${error instanceof Error ? error.message : String(error)}`,
  );
  process.exit(1);
}
