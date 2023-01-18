import { MigrateCommandArgument } from "../src/commands";
import migrate from "../src/commands/migrate";
import { setupTestFixtures, spyExit } from "./test-utils";
import * as checkGitStatus from "../src/utils/checkGitStatus";
import * as getCurrentVersion from "../src/commands/migrate/steps/getCurrentVersion";
import * as getLatestVersion from "../src/commands/migrate/steps/getLatestVersion";
import * as getTurboUpgradeCommand from "../src/commands/migrate/steps/getTurboUpgradeCommand";
import * as workspaceImplementation from "../src/utils/getWorkspaceImplementation";
import * as getPackageManagerVersion from "../src/utils/getPackageManagerVersion";

describe("migrate", () => {
  const mockExit = spyExit();
  const { useFixture } = setupTestFixtures({ test: "migrate" });

  it("migrates from 1.0.0 to 1.7.0", async () => {
    const { root, readJson } = useFixture({
      fixture: "old-turbo",
    });

    const packageManager = "pnpm";
    const packageManagerVersion = "1.2.3";

    // setup mocks
    const mockedCheckGitStatus = jest
      .spyOn(checkGitStatus, "default")
      .mockReturnValue(undefined);
    const mockedGetCurrentVersion = jest
      .spyOn(getCurrentVersion, "default")
      .mockReturnValue("1.0.0");
    const mockedGetLatestVersion = jest
      .spyOn(getLatestVersion, "default")
      .mockResolvedValue("1.7.0");
    const mockedGetTurboUpgradeCommand = jest
      .spyOn(getTurboUpgradeCommand, "default")
      .mockReturnValue("pnpm install -g turbo@latest");
    const mockedGetPackageManagerVersion = jest
      .spyOn(getPackageManagerVersion, "default")
      .mockReturnValue(packageManagerVersion);
    const mockedGetWorkspaceImplementation = jest
      .spyOn(workspaceImplementation, "default")
      .mockReturnValue(packageManager);

    await migrate(root as MigrateCommandArgument, {
      force: false,
      dry: false,
      print: false,
      install: false,
    });

    expect(readJson("package.json")).toStrictEqual({
      dependencies: {},
      devDependencies: {
        turbo: "1.0.0",
      },
      name: "no-turbo-json",
      packageManager: "pnpm@1.2.3",
      version: "1.0.0",
    });
    expect(readJson("turbo.json")).toStrictEqual({
      $schema: "https://turbo.build/schema.json",
      pipeline: {
        build: {
          outputs: [".next/**"],
        },
        dev: {
          cache: false,
          outputs: ["dist/**", "build/**"],
        },
        lint: {},
      },
    });

    // verify mocks were called
    expect(mockedCheckGitStatus).toHaveBeenCalled();
    expect(mockedGetCurrentVersion).toHaveBeenCalled();
    expect(mockedGetLatestVersion).toHaveBeenCalled();
    expect(mockedGetTurboUpgradeCommand).toHaveBeenCalled();
    expect(mockedGetPackageManagerVersion).toHaveBeenCalled();
    expect(mockedGetWorkspaceImplementation).toHaveBeenCalled();

    // restore mocks
    mockedCheckGitStatus.mockRestore();
    mockedGetCurrentVersion.mockRestore();
    mockedGetLatestVersion.mockRestore();
    mockedGetTurboUpgradeCommand.mockRestore();
    mockedGetPackageManagerVersion.mockRestore();
    mockedGetWorkspaceImplementation.mockRestore();
  });

  it("next version can be passed as an option", async () => {
    const { root, readJson } = useFixture({
      fixture: "old-turbo",
    });

    const packageManager = "pnpm";
    const packageManagerVersion = "1.2.3";

    // setup mocks
    const mockedCheckGitStatus = jest
      .spyOn(checkGitStatus, "default")
      .mockReturnValue(undefined);
    const mockedGetCurrentVersion = jest
      .spyOn(getCurrentVersion, "default")
      .mockReturnValue("1.0.0");
    const mockedGetTurboUpgradeCommand = jest
      .spyOn(getTurboUpgradeCommand, "default")
      .mockReturnValue("pnpm install -g turbo@latest");
    const mockedGetPackageManagerVersion = jest
      .spyOn(getPackageManagerVersion, "default")
      .mockReturnValue(packageManagerVersion);
    const mockedGetWorkspaceImplementation = jest
      .spyOn(workspaceImplementation, "default")
      .mockReturnValue(packageManager);

    await migrate(root as MigrateCommandArgument, {
      force: false,
      dry: false,
      print: false,
      install: false,
      to: "1.7.0",
    });

    expect(readJson("package.json")).toStrictEqual({
      dependencies: {},
      devDependencies: {
        turbo: "1.0.0",
      },
      name: "no-turbo-json",
      packageManager: "pnpm@1.2.3",
      version: "1.0.0",
    });
    expect(readJson("turbo.json")).toStrictEqual({
      $schema: "https://turbo.build/schema.json",
      pipeline: {
        build: {
          outputs: [".next/**"],
        },
        dev: {
          cache: false,
          outputs: ["dist/**", "build/**"],
        },
        lint: {},
      },
    });

    // verify mocks were called
    expect(mockedCheckGitStatus).toHaveBeenCalled();
    expect(mockedGetCurrentVersion).toHaveBeenCalled();
    expect(mockedGetTurboUpgradeCommand).toHaveBeenCalled();
    expect(mockedGetPackageManagerVersion).toHaveBeenCalled();
    expect(mockedGetWorkspaceImplementation).toHaveBeenCalled();

    // restore mocks
    mockedCheckGitStatus.mockRestore();
    mockedGetCurrentVersion.mockRestore();
    mockedGetTurboUpgradeCommand.mockRestore();
    mockedGetPackageManagerVersion.mockRestore();
    mockedGetWorkspaceImplementation.mockRestore();
  });

  it("current version can be passed as an option", async () => {
    const { root, readJson } = useFixture({
      fixture: "old-turbo",
    });

    const packageManager = "pnpm";
    const packageManagerVersion = "1.2.3";

    // setup mocks
    const mockedCheckGitStatus = jest
      .spyOn(checkGitStatus, "default")
      .mockReturnValue(undefined);
    const mockedGetLatestVersion = jest
      .spyOn(getLatestVersion, "default")
      .mockResolvedValue("1.7.0");
    const mockedGetTurboUpgradeCommand = jest
      .spyOn(getTurboUpgradeCommand, "default")
      .mockReturnValue("pnpm install -g turbo@latest");
    const mockedGetPackageManagerVersion = jest
      .spyOn(getPackageManagerVersion, "default")
      .mockReturnValue(packageManagerVersion);

    const mockedGetWorkspaceImplementation = jest
      .spyOn(workspaceImplementation, "default")
      .mockReturnValue(packageManager);

    await migrate(root as MigrateCommandArgument, {
      force: false,
      dry: false,
      print: false,
      install: false,
      from: "1.0.0",
    });

    expect(readJson("package.json")).toStrictEqual({
      dependencies: {},
      devDependencies: {
        turbo: "1.0.0",
      },
      name: "no-turbo-json",
      packageManager: "pnpm@1.2.3",
      version: "1.0.0",
    });
    expect(readJson("turbo.json")).toStrictEqual({
      $schema: "https://turbo.build/schema.json",
      pipeline: {
        build: {
          outputs: [".next/**"],
        },
        dev: {
          cache: false,
          outputs: ["dist/**", "build/**"],
        },
        lint: {},
      },
    });

    // verify mocks were called
    expect(mockedCheckGitStatus).toHaveBeenCalled();
    expect(mockedGetLatestVersion).toHaveBeenCalled();
    expect(mockedGetTurboUpgradeCommand).toHaveBeenCalled();
    expect(mockedGetPackageManagerVersion).toHaveBeenCalled();
    expect(mockedGetWorkspaceImplementation).toHaveBeenCalled();

    // restore mocks
    mockedCheckGitStatus.mockRestore();
    mockedGetLatestVersion.mockRestore();
    mockedGetTurboUpgradeCommand.mockRestore();
    mockedGetPackageManagerVersion.mockRestore();
    mockedGetWorkspaceImplementation.mockRestore();
  });

  it("exits if the current version is the same as the new version", async () => {
    const { root, readJson } = useFixture({
      fixture: "old-turbo",
    });

    // setup mocks
    const mockedCheckGitStatus = jest
      .spyOn(checkGitStatus, "default")
      .mockReturnValue(undefined);
    const mockedGetCurrentVersion = jest
      .spyOn(getCurrentVersion, "default")
      .mockReturnValue("1.7.0");
    const mockedGetLatestVersion = jest
      .spyOn(getLatestVersion, "default")
      .mockResolvedValue("1.7.0");

    await migrate(root as MigrateCommandArgument, {
      force: false,
      dry: false,
      print: false,
      install: false,
    });

    expect(mockExit.exit).toHaveBeenCalledWith(0);

    // verify mocks were called
    expect(mockedCheckGitStatus).toHaveBeenCalled();
    expect(mockedGetCurrentVersion).toHaveBeenCalled();
    expect(mockedGetLatestVersion).toHaveBeenCalled();

    // restore mocks
    mockedCheckGitStatus.mockRestore();
    mockedGetCurrentVersion.mockRestore();
    mockedGetLatestVersion.mockRestore();
  });

  it("exits if current version is not passed and cannot be inferred", async () => {
    const { root } = useFixture({
      fixture: "old-turbo",
    });

    // setup mocks
    const mockedCheckGitStatus = jest
      .spyOn(checkGitStatus, "default")
      .mockReturnValue(undefined);
    const mockedGetCurrentVersion = jest
      .spyOn(getCurrentVersion, "default")
      .mockReturnValue(undefined);

    await migrate(root as MigrateCommandArgument, {
      force: false,
      dry: false,
      print: false,
      install: false,
    });

    expect(mockExit.exit).toHaveBeenCalledWith(1);

    // verify mocks were called
    expect(mockedCheckGitStatus).toHaveBeenCalled();
    expect(mockedGetCurrentVersion).toHaveBeenCalled();

    // restore mocks
    mockedCheckGitStatus.mockRestore();
    mockedGetCurrentVersion.mockRestore();
  });

  it("exits if latest version is not passed and cannot be inferred", async () => {
    const { root } = useFixture({
      fixture: "old-turbo",
    });

    // setup mocks
    const mockedCheckGitStatus = jest
      .spyOn(checkGitStatus, "default")
      .mockReturnValue(undefined);
    const mockedGetCurrentVersion = jest
      .spyOn(getCurrentVersion, "default")
      .mockReturnValue('1.5.0');
      const mockedGetLatestVersion = jest
      .spyOn(getLatestVersion, "default")
      .mockResolvedValue(undefined);

    await migrate(root as MigrateCommandArgument, {
      force: false,
      dry: false,
      print: false,
      install: false,
    });

    expect(mockExit.exit).toHaveBeenCalledWith(1);

    // verify mocks were called
    expect(mockedCheckGitStatus).toHaveBeenCalled();
    expect(mockedGetCurrentVersion).toHaveBeenCalled();
    expect(mockedGetLatestVersion).toHaveBeenCalled();

    // restore mocks
    mockedCheckGitStatus.mockRestore();
    mockedGetCurrentVersion.mockRestore();
    mockedGetLatestVersion.mockRestore();
  });

  it("exits if invalid directory is passed", async () => {
    const { root } = useFixture({
      fixture: "old-turbo",
    });

    // setup mocks
    const mockedCheckGitStatus = jest
      .spyOn(checkGitStatus, "default")
      .mockReturnValue(undefined);

    await migrate('~/path/that/does/not/exist' as MigrateCommandArgument, {
      force: false,
      dry: false,
      print: false,
      install: false,
    });

    expect(mockExit.exit).toHaveBeenCalledWith(1);

    // verify mocks were called
    expect(mockedCheckGitStatus).toHaveBeenCalled();

    // restore mocks
    mockedCheckGitStatus.mockRestore();
  });
});
