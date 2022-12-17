#!/usr/bin/env node

import fs from "node:fs";
import { join } from "node:path";
import { executables } from "./executables.mjs";
import child_process from "node:child_process";
import { fileURLToPath } from "node:url";

const __dirname = fileURLToPath(new URL(".", import.meta.url));

(async function () {
  const executable = executables.get(`${process.platform}-${process.arch}`);
  const issuesURL = "https://github.com/bjesuiter/concurrently-rust/issues";

  if (!executable) {
    const supportedPlatforms = executables.keys.join(", ");
    throw new Error(
      `Your platform (${process.platform}, ${process.arch}) is currently not supported!
      Platforms supported: ${supportedPlatforms}.
      You can raise an issue here and ask for support: ${issuesURL}`
    );
  }

  let executablePath = join(
    __dirname,
    executable.platform,
    executable.arch,
    executable.executableName
  );

  if (!fs.existsSync(executablePath))
    throw new Error(`Deno Executable not found at ${executablePath}. Something is wrong with this install.
  Please raise an issue at: ${issuesURL}`);

  const p = child_process.spawnSync(executablePath, process.argv.slice(2), {
    cwd: process.cwd(),
    stdio: "inherit",
    shell: false,
  });

  if (p.error) throw new Error(p.error);
})();
