import { isLinux } from "https://deno.land/std@0.162.0/_util/os.ts";
import { join } from "https://deno.land/std@0.168.0/path/mod.ts";
import { ensureDir } from "https://deno.land/std@0.168.0/fs/mod.ts";

/**
 * Can only be run on macos or isLinux, bc it sets executable flag on deno.js
 *
 * Note: Paths are measured from root of repo
 */
export async function copyAssets(outPath?: string) {
  // make sure, that the outPath exists
  if (!outPath) {
    outPath = `dist/`;
  }
  await ensureDir(outPath);
  await ensureDir(join(outPath, "bin"));

  const sourcePath = "./npm-assets";
  await Deno.copyFile(join(sourcePath, "index.js"), join(outPath, "index.js"));
  await Deno.copyFile(
    join(sourcePath, "bin/run.mjs"),
    join(outPath, "bin/run.mjs"),
  );

  if (isLinux) {
    await Deno.chmod(join(outPath, "bin", "run.mjs"), 0o775);
  }

  await Deno.copyFile(
    join(sourcePath, "bin/executables.mjs"),
    join(outPath, "bin/executables.mjs"),
  );

  // Copy release rust binaries from target folder
  await (ensureDir(join(outPath, "bin/darwin")));
  await (ensureDir(join(outPath, "bin/windows")));
  await Deno.copyFile(
    `target/release/concurrently`,
    join(outPath, "bin/darwin/concurrently"),
  );
  await Deno.copyFile(
    `target/x86_64-pc-windows-gnu/release/concurrently.exe`,
    join(outPath, "bin/windows/concurrently.exe"),
  );

  console.info(`Copied assets!`);
}
