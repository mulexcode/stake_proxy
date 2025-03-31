import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { bs58 } from "@coral-xyz/anchor/dist/cjs/utils/bytes";
import {Buffer} from "buffer";
import Wallet from "ethereumjs-wallet";
import {randomBytes} from "crypto";
import BN from "bn.js";
import {SolanaBridge} from "../target/types/solana_bridge";
import {createMint, getOrCreateAssociatedTokenAccount} from "@solana/spl-token";

// Configure the client to use the local cluster.
anchor.setProvider(anchor.AnchorProvider.env());

function uint8ToNumberArray(u8:Uint8Array) : number[] {
    const arr: number[] = [];
    for (let i = 0; i < u8.length; i++) {
        arr.push(u8[i]);
    }
    return arr;
}

async function cash_out() {
    anchor.setProvider(anchor.AnchorProvider.env());
    const program = anchor.workspace.SolanaBridge as Program<SolanaBridge>;
    const [nativeVault, ] = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("native_token_vault")], program.programId);
    console.log("nativeVault: ", nativeVault.toString())
    const payer = anchor.Wallet.local().payer;
    const [systemConfig, ]= anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("system_config")], program.programId);
    // const stakeMint = new anchor.web3.PublicKey("Mix1111111111111111111111111111111111111111");
    // const ethAddress = "2f4F09b722a6e5b77bE17c9A99c785Fa7035a09f";
    const toChain = 103;
    const [chainConfig, ] = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("chain_config"), new BN(toChain).toArrayLike(Buffer, 'le', 8)], program.programId);
    // const [solChainConfig, ] = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("chain_config"), new BN(101).toArrayLike(Buffer, 'le', 8)], program.programId);
    // const [tokenConfig, ] = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("token_config"), Buffer.from("mix")], program.programId);
    // const [solTokenConfig, ] = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("token_config"), Buffer.from("sol")], program.programId);

    const tx = await program.methods.cashOutSol(
        new BN(toChain),
        payer.publicKey,
        new BN(1000000),
    ).accountsPartial({
        config: systemConfig,
        payer: payer.publicKey,
        chainConfig: chainConfig,
    }).signers([payer]).rpc();
    console.log(tx);
}

cash_out()
    .then(() => process.exit(0))
  .catch((err) => {
    console.error(err);
    process.exit(1);
  });
