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
    console.log("nativeVault: ", nativeVault.toString())
    const payer = anchor.Wallet.local().payer;
    const [systemConfig, ]= anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("system_config")], program.programId);

    const ethAddress = "bd18C7721776CE9C9e4dA7C976332D1070dc8ACD";
    const [chainConfig, ] = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("chain_config"), new BN(1).toArrayLike(Buffer, 'le', 8)], program.programId);
    const [solChainConfig, ] = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("chain_config"), new BN(101).toArrayLike(Buffer, 'le', 8)], program.programId);
    const [tokenConfig, ] = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("token_config"), Buffer.from("test")], program.programId);
    const [solTokenConfig, ] = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("token_config"), Buffer.from("sol")], program.programId);
    const anotherPayer = anchor.web3.Keypair.generate();


    await program.methods.updateManager(payer.publicKey, uint8ToNumberArray(Buffer.from(ethAddress, 'hex'))).accountsPartial({
        manager: payer.publicKey,
    }).signers([payer]).rpc();
    //
    // await program.methods.enableChain(new BN(103)).accountsPartial({
    //     payer: payer.publicKey,
    // }).signers([payer]).rpc();

    // await program.methods.enableToken("mix").accountsPartial({
    //     payer: payer.publicKey,
    //     tokenMint: "Mix1111111111111111111111111111111111111111",
    // }).signers([payer]).rpc();

    // await program.methods.enableToken("sol").accountsPartial({
    //     payer: payer.publicKey,
    //     tokenMint: "Mix1111111111111111111111111111111111111111",
    // }).signers([payer]).rpc();
}

deploy()
  .then(() => process.exit(0))
  .catch((err) => {
    console.error(err);
    process.exit(1);
  });
