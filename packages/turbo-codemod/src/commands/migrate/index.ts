import chalk from "chalk";
import os from "os";
import inquirer from "inquirer";

import getCurrentVersion from "./steps/getCurrentVersion";
import getLatestVersion from "./steps/getLatestVersion";
import getCodemodsForMigration from "./steps/getTransformsForMigration";
import checkGitStatus from "../../utils/checkGitStatus";
import directoryInfo from "../../utils/directoryInfo";
import getTurboUpgradeCommand from "./steps/getTurboUpgradeCommand";
import { execSync } from "child_process";
import Runner from "../../runner/Runner";
import type { MigrateCommandArgument, MigrateCommandOptions } from "./types";

/*
Migration is done in 4 steps:
  -- gather information
  1. find the version (x) of turbo to migrate from (if not specified)
  2. find the version (y) of turbo to migrate to (if not specified)
  3. determine which codemods need to be run to move from version x to version y
  -- action
  4. execute the codemods (serially, and in order)
  5. update the turbo version (optionally)

*/
export default async function migrate(
  directory: MigrateCommandArgument,
  options: MigrateCommandOptions
) {
  // check git status
  if (!options.dry) {
    checkGitStatus(options.force);
  }

  const answers = await inquirer.prompt<{
    directoryInput?: string;
  }>([
    {
      type: "input",
      name: "directoryInput",
      message: "Where is the root of the repo where the transform should run?",
      when: !directory,
      default: ".",
      validate: (directory: string) => {
        const { exists, absolute } = directoryInfo({ directory });
        if (exists) {
          return true;
        } else {
          return `Directory ${chalk.dim(`(${absolute})`)} does not exist`;
        }
      },
      filter: (directory: string) => directory.trim(),
    },
  ]);

  const { directoryInput: selectedDirectory = directory as string } = answers;
  const { exists, absolute: root } = directoryInfo({
    directory: selectedDirectory,
  });
  if (!exists) {
    console.error(`Directory ${chalk.dim(`(${root})`)} does not exist`);
    return process.exit(1);
  }

  // step 1
  const fromVersion = getCurrentVersion(selectedDirectory, options);
  if (!fromVersion) {
    console.error(`Unable to infer the version of turbo being used by ${directory}`);
    return process.exit(1);
  }

  // step 2
  const toVersion = await getLatestVersion(options);
  if (!toVersion) {
    console.error(`Unable to fetch the latest version of turbo`);
    return process.exit(1);
  }

  if (fromVersion === toVersion) {
    console.log(
      `Nothing to do, current version (${chalk.bold(
        fromVersion
      )}) is the same as the requested version (${chalk.bold(toVersion)})`
    );
    
    return process.exit(0);
  }

  // step 3
  const codemods = await getCodemodsForMigration({ fromVersion, toVersion });
  if (codemods.length === 0) {
    console.log(
      `No codemods required to migrate from ${fromVersion} to ${toVersion}`
    );
  }

  // step 4
  console.log(
    os.EOL,
    `Upgrading turbo from ${chalk.bold(fromVersion)} to ${chalk.bold(
      toVersion
    )}`
  );
  const result = codemods.map((codemod, idx) => {
    console.log(
      os.EOL,
      `(${idx + 1}/${codemods.length}) ${chalk.bold(
        `Running ${codemod.value}`
      )}`
    );

    return codemod.transformer({ root: selectedDirectory, options });
  });

  console.log(os.EOL, chalk.bold("File Change Summary"));
  result.forEach(Runner.logResults);

  // step 5
  const upgradeCommand = getTurboUpgradeCommand({
    directory: selectedDirectory,
    to: options.to,
  });

  if (options.install) {
    if (upgradeCommand) {
      console.log(os.EOL, `Upgrading turbo with ${chalk.bold(upgradeCommand)}`);
      execSync(upgradeCommand, { cwd: selectedDirectory });
    } else {
      console.log("Unable to determine turbo upgrade command");
    }
  } else {
    if (upgradeCommand) {
      console.log(os.EOL, `Upgrade turbo with ${chalk.bold(upgradeCommand)}`);
    } else {
      console.log("Unable to determine turbo upgrade command");
    }
  }

  console.log(os.EOL, "Migration completed!");
}
