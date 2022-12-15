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
Deno.exit();

export const packageJson = {
  name: "deno-npm",
  version: "",
  description:
    "An inofficial distribution of the deno binary, a secure runtime for JavaScript and TypeScript (Offline-Install), based on deno-bin",
  type: "module",
  // Not needded for type module
  // main: "index.js",
  // Note: Automatically added files:
  // main script (if applicable), Readme.md, package.json
  files: [
    "bin",
  ],
  bin: {
    "deno": "./bin/deno.js",
    "deno-bin-offline": "./bin/deno.js",
    "deno-npm": "./bin/deno.js",
  },
  scripts: {
    "start": "node ./bin/deno.js",
    "deno-version": "./bin/deno.js --version",
  },
  repository: {
    type: "git",
    url: "git+https://github.com/codemonument/deno-bin-offline.git",
  },
  keywords: [
    "deno",
  ],
  author: "Benjamin Jesuiter",
  license: "MIT",
  bugs: {
    url: "https://github.com/codemonument/deno-bin-offline/issues",
  },
  homepage: "https://github.com/codemonument/deno-bin-offline#readme",
  dependencies: {},
  devDependencies: {
    "@types/node": "^18.11.9",
  },
};

export async function generatePackageJson(outPath?: string) {
  if (!outPath) {
    outPath = `dist/`;
  }

  await Deno.writeTextFile(
    join(outPath, "package.json"),
    JSON.stringify(packageJson, null, "\t"),
  );

  console.info(`Generated package.json!`);
}
