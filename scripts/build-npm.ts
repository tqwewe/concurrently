// This script file should be run with deno to generate the npm package output
// deno run --allow-run --allow-read=Cargo.toml npm-builder/build.ts

import { copyAssets } from "../npm-builder/copyAssets.task.ts";
import { generatePackageJson } from "../npm-builder/packageJson.template.ts";
import { ensureDir } from "https://deno.land/std@0.168.0/fs/mod.ts";
import { join } from "https://deno.land/std@0.168.0/path/mod.ts";

const outPath = "dist/";
await ensureDir(outPath);
await ensureDir(join(outPath, "bin"));

await generatePackageJson(outPath);
await copyAssets(outPath);
