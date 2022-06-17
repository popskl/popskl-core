import yargs from "yargs";
import { hideBin } from "yargs/helpers";
import fs from "fs";
import sha256 from "sha256";
import bs58 from "bs58";

yargs(hideBin(process.argv))
  .command(
    "hash-code <wasm-file>",
    "Created hash of provided binary. Used to check integrity of deployed code.",
    (yargs) =>
      yargs.positional("wasm-file", {
        type: "string",
        desc: "Path to contract wasm",
      }),
    (args) => {
      const binary = fs.readFileSync(args.wasmFile);
      console.log(bs58.encode(sha256(binary, { asBytes: true })));
    }
  )
  .demandCommand()
  .parse();
