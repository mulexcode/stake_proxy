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

async function update_nonce() {
    anchor.setProvider(anchor.AnchorProvider.env());
    const program = anchor.workspace.SolanaBridge as Program<SolanaBridge>;
    const payer = anchor.Wallet.local().payer;
    const toChain = 11155111;
    const [chainConfig, ] = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("chain_config"), new BN(toChain).toArrayLike(Buffer, 'le', 8)], program.programId);

    const tx = await program.methods.updateNonce(new BN(11155111), new BN(0), true).accountsPartial({
        manager: payer.publicKey,
        chainConfig
    }).signers([payer]).rpc();
    console.log("tx", tx);
}

update_nonce()
    .then(() => process.exit(0))
    .catch((err) => {
        console.error(err);
        process.exit(1);
    });
