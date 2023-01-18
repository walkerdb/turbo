import path from "path";
import { execSync, ExecSyncOptions } from "child_process";
import { MigrateCommandOptions } from "../types";
import { existsSync } from "fs-extra";
import getWorkspaceImplementation from "../../../utils/getWorkspaceImplementation";

function exec(
  command: string,
  opts: ExecSyncOptions,
  fallback?: string
): string | undefined {
  try {
    const rawResult = execSync(command, opts);
    return rawResult.toString("utf8").trim();
  } catch (err) {
    return fallback || undefined;
  }
}

function getCurrentVersion(
  directory: string,
  opts: MigrateCommandOptions
): string | undefined {
  const { from } = opts;
  if (from) {
    return from;
  }

  // try global first
  const turboVersionFromGlobal = exec(`turbo --version`, { cwd: directory });

  if (turboVersionFromGlobal) {
    return turboVersionFromGlobal;
  }

  // try to use the package manager to find the version
  const packageManager = getWorkspaceImplementation(directory);
  if (packageManager) {
    if (packageManager === "yarn") {
      return exec(`yarn turbo --version`, { cwd: directory });
    }
    if (packageManager === "pnpm") {
      return exec(`pnpm turbo --version`, { cwd: directory });
    } else {
      // this doesn't work for npm, so manually build the binary path
      const turboBin = path.join(directory, "node_modules", ".bin", "turbo");
      if (existsSync(turboBin)) {
        return exec(`${turboBin} --version`, { cwd: directory });
      }
    }
  }

  // unable to determine local version,
  return undefined;
}

export default getCurrentVersion;
