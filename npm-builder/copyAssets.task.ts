import { isLinux } from "https://deno.land/std@0.162.0/_util/os.ts";
import { join } from "https://deno.land/std@0.168.0/path/mod.ts";
import { ensureDir } from "https://deno.land/std@0.168.0/fs/mod.ts";

/**
 * Can only be run on macos or isLinux, bc it sets executable flag on deno.js
 *
 * Note: Paths are measured from root of repo
 */
export async function copyAssets(outPath?: string) {
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

  console.info(`Copied assets!`);
}
