import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { bs58 } from "@coral-xyz/anchor/dist/cjs/utils/bytes";
import {Buffer} from "buffer";
import Wallet from "ethereumjs-wallet";
import {randomBytes} from "crypto";
import BN from "bn.js";
import {SolanaBridge} from "../target/types/solana_bridge";
import {createMint} from "@solana/spl-token";

// Configure the client to use the local cluster.
anchor.setProvider(anchor.AnchorProvider.env());

function uint8ToNumberArray(u8:Uint8Array) : number[] {
    const arr: number[] = [];
    for (let i = 0; i < u8.length; i++) {
        arr.push(u8[i]);
    }
    return arr;
}

async function deploy() {
    anchor.setProvider(anchor.AnchorProvider.env());
    const program = anchor.workspace.SolanaBridge as Program<SolanaBridge>;
    const [nativeVault, ] = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("native_token_vault")], program.programId);

    const payer = anchor.Wallet.local().payer;

    const ethAddress = "248449e216aa3f7c4700969fcc5b258b5335170b";
    //const ethAddress = "bd18C7721776CE9C9e4dA7C976332D1070dc8ACD"; // TEST


    const initializeTx = await program.methods.initialize(new BN(103), payer.publicKey, uint8ToNumberArray(Buffer.from(ethAddress, 'hex'))).accountsPartial({
        nativeTokenVault: nativeVault,
        payer: payer.publicKey,
    }).instruction();
    //
    const enable8888 = await program.methods.enableChain(new BN(8888)).accountsPartial({
        payer: payer.publicKey,
    }).instruction();

    const enableMix = await program.methods.enableToken("mix").accountsPartial({
        payer: payer.publicKey,
        tokenMint: "MixSFCPowwkjcBKjBhEsQQkKtvpy7kMVkEEA2mF46JM",
    }).instruction();

    const enableSol = await program.methods.enableToken("sol").accountsPartial({
        payer: payer.publicKey,
        tokenMint: "So11111111111111111111111111111111111111112",
    }).instruction();

    let tx = new anchor.web3.Transaction().add(
        initializeTx,
        enable8888,
        enableMix,
        enableSol,
    );
    console.log(
        `txhash: ${await anchor.getProvider().sendAndConfirm(tx, [payer])}`,
    );
}

deploy()
  .then(() => process.exit(0))
  .catch((err) => {
    console.error(err);
    process.exit(1);
  });
