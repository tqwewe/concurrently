// This script file should be run with deno to generate the npm package output
// deno run --allow-run --allow-read=Cargo.toml npm-builder/build.ts

import { generatePackageJson } from "./packageJson.template.ts";

await generatePackageJson();
