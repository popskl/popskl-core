import yargs from "yargs/yargs";
import { hideBin } from "yargs/helpers";
import cryptoRandomString from "crypto-random-string";
import keccak256 from "keccak256";
import bs58 from "bs58";

yargs(hideBin(process.argv))
  .command(
    "secret",
    "Generate random secret",
    (yargs) => yargs,
    () => {
      console.log(cryptoRandomString({ length: 32, type: "ascii-printable" }));
    }
  )
  .command(
    "hash <location> <secret>",
    "Hash provided location and secret",
    (yargs) =>
      yargs
        .positional("location", {
          type: "string",
          desc: "Location you want to store",
        })
        .positional("secret", {
          type: "string",
          desc: "Secret to include in hash",
        }),
    (args) => {
      console.log(bs58.encode(keccak256(`${args.location}|${args.secret}`)));
    }
  )
  .command(
    "bs58 <value>",
    "Encode value as bs58",
    (yargs) =>
      yargs.positional("value", { type: "string", desc: "Value to encode" }),
    (args) => {
      console.log(bs58.encode(new TextEncoder().encode(args.value)));
    }
  )
  .demandCommand()
  .parse();
