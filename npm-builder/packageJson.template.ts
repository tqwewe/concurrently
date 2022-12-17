import { ensureDir } from "https://deno.land/std@0.168.0/fs/mod.ts";
import { join } from "https://deno.land/std@0.168.0/path/mod.ts";
import { parse } from "https://deno.land/std@0.168.0/encoding/toml.ts";
import { z } from "https://deno.land/x/zod@v3.20.2/mod.ts";

// get current version from Cargo.toml (path measured from root of repo)
const cargoTomlString = await Deno.readTextFile("Cargo.toml");
const cargoTomlObj = parse(cargoTomlString);

const PartialCargoTomlSchema = z.object({
  package: z.object({
    // zod schema for validating semVer strings
    version: z.string().regex(/^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/),
  }),
});

const version = PartialCargoTomlSchema.parse(cargoTomlObj).package.version;

// console.log(cargoToml.package.version);
console.log(version);

export const packageJson = {
  name: "concurrently-rust",
  version: "",
  description:
    "A distribution of https://github.com/bjesuiter/concurrently-rust, a Rust implementation of a similar concept to the npm concurrently package.",
  type: "module",
  // Not needded for type module
  // main: "index.js",
  // Note: Automatically added files:
  // main script (if applicable), Readme.md, package.json
  files: [
    "bin",
  ],
  bin: {
    "concurrently": "./bin/run.mjs",
    "concurrently-rust": "./bin/run.mjs",
  },
  scripts: {
    "start": "node ./bin/run.mjs",
    "test": "./bin/run.mjs --help",
  },
  repository: {
    type: "git",
    url: "git+https://github.com/bjesuiter/concurrently-rust",
  },
  keywords: [
    "rust",
    "concurrently",
    "cli",
    "cli-tool",
    "cli-rust",
    "cli-tool-rust",
  ],
  author: "Benjamin Jesuiter",
  license: "MIT",
  bugs: {
    url: "https://github.com/bjesuiter/concurrently-rust/issues",
  },
  homepage: "https://github.com/bjesuiter/concurrently-rust#readme",
  dependencies: {},
  devDependencies: {
    "@types/node": "^18.11.9",
  },
};

export async function generatePackageJson(outPath?: string) {
  if (!outPath) {
    outPath = `dist/`;
  }

  await ensureDir(outPath);
  await Deno.writeTextFile(
    join(outPath, "package.json"),
    JSON.stringify(packageJson, null, "\t"),
    { create: true },
  );

  console.info(`Generated package.json!`);
}
