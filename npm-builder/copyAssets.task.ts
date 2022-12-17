import { isLinux } from "https://deno.land/std@0.162.0/_util/os.ts";

/**
 * Can only be run on macos or isLinux, bc it sets executable flag on deno.js
 *
 * Note: Paths are measured from root of repo
 */
export async function copyAssets() {
  await Deno.copyFile(`./npm-assets/index.js`, `./dist/index.js`);
  await Deno.copyFile(`./npm-assets/bin/run.mjs`, `./dist/bin/run.mjs`);

  if (isLinux) {
    await Deno.chmod(`./dist/bin/run.mjs`, 0o775);
  }

  await Deno.copyFile(
    `./assets/bin/executables.mjs`,
    `./dist/bin/executables.mjs`,
  );

  console.info(`Copied assets!`);
}
